use outlook_pst::{ltp::prop_context::PropertyValue, messaging::{folder::Folder}, ndb::node_id::NodeId, open_store};
use std::rc::Rc;
use std::time::Instant;

pub fn run_bench(args: crate::args::BenchArgs) -> anyhow::Result<()> {
    let start_total = Instant::now();
    let files = crate::iterate_emails::collect_pst_files(&args.input)?;
    if files.is_empty() { println!("No .pst files found."); return Ok(()); }
    // Aggregates
    let mut total_messages = 0usize;
    let mut header_only_resolved = 0usize; // recipients satisfied by headers
    let mut needed_table = 0usize;        // headers empty, table would be needed
    let mut table_forced = 0usize;        // count of times we actually traversed table (force or fallback)
    let mut timing_header_us: u128 = 0;
    let mut timing_table_us: u128 = 0;
    let mut timing_body_us: u128 = 0; // just to observe unrelated cost

    for pst_path in files {
        let store = open_store(&pst_path)?;
        let ipm_sub = match store.properties().ipm_sub_tree_entry_id() { Ok(v) => v, Err(e) => { eprintln!("Skipping store '{}': {}", pst_path.display(), e); continue; } };
        let root = match store.open_folder(&ipm_sub) { Ok(f) => f, Err(e) => { eprintln!("Skipping store '{}': {}", pst_path.display(), e); continue; } };
        // DFS stack of (folder, path)
        let mut stack: Vec<(Rc<dyn Folder>, String)> = vec![(root, String::new())];
        while let Some((folder, path)) = stack.pop() {
            if let Some(ht) = folder.hierarchy_table() {
                for row in ht.rows_matrix() {
                    let node_id = NodeId::from(u32::from(row.id()));
                    if let Ok(entry_id) = store.properties().make_entry_id(node_id) {
                        if let Ok(sub) = store.open_folder(&entry_id) {
                            let name = sub.properties().display_name().unwrap_or_else(|_| "?".to_string());
                            let new_path = if path.is_empty() { name } else { format!("{}/{}", path, name) };
                            stack.push((sub, new_path));
                        }
                    }
                }
            }
            if let Some(ct) = folder.contents_table() {
                for row in ct.rows_matrix() {
                    total_messages += 1;
                    let node_id = NodeId::from(u32::from(row.id()));
                    let entry_id = match store.properties().make_entry_id(node_id) { Ok(e) => e, Err(_) => continue };
                    let message = match store.open_message(&entry_id, None) { Ok(m) => m, Err(_) => continue };
                    let props = message.properties();
                    // Measure headers path
                    let t0 = Instant::now();
                    let header_text = props.get(0x007D).and_then(|v| match v { PropertyValue::String8(s) => Some(s.to_string()), PropertyValue::Unicode(u) => Some(u.to_string()), _ => None });
                    let headers = header_text.map(|t| crate::iterate_emails::parse_transport_headers(&t));
                    let (mut to, mut cc, mut bcc) = (Vec::new(), Vec::new(), Vec::new());
                    if let Some(h) = &headers {
                        to = h.to.as_ref().map(|s| crate::iterate_emails::parse_recipients_from_header(s)).unwrap_or_default();
                        cc = h.cc.as_ref().map(|s| crate::iterate_emails::parse_recipients_from_header(s)).unwrap_or_default();
                        bcc = h.bcc.as_ref().map(|s| crate::iterate_emails::parse_recipients_from_header(s)).unwrap_or_default();
                    }
                    timing_header_us += t0.elapsed().as_micros();
                    let headers_present = !(to.is_empty() && cc.is_empty() && bcc.is_empty());
                    if headers_present { header_only_resolved += 1; }
                    let mut did_table = false;
                    if args.force_table || (!headers_present && args.lazy_fallback) {
                        let t1 = Instant::now();
                        let rt = message.recipient_table();
                        let info = rt.context();
                        for row in rt.rows_matrix() {
                            // read columns we care about (recipient type 0x0C15, display 0x3001, smtp 0x39FE, email 0x3003)
                            let cols = match row.columns(info) { Ok(c) => c, Err(_) => continue };
                            // Keep simple presence count; we don't need to build full strings for benchmark unless missing
                            // Just trigger read to simulate work
                            for (col_def, value) in info.columns().iter().zip(cols) {
                                if matches!(col_def.prop_id(), 0x0C15 | 0x3001 | 0x39FE | 0x3003) {
                                    if let Some(valref) = value.as_ref() { let _ = rt.read_column(valref, col_def.prop_type()); }
                                }
                            }
                        }
                        timing_table_us += t1.elapsed().as_micros();
                        did_table = true;
                        if !headers_present { needed_table += 1; }
                    } else if !headers_present { needed_table += 1; }
                    if did_table { table_forced += 1; }
                    // Optional body measure (to contextualize cost of recipients vs body extraction)
                    let t2 = Instant::now();
                    let _ = crate::iterate_emails::extract_plain_body(message.as_ref());
                    timing_body_us += t2.elapsed().as_micros();
                }
            }
        }
    }

    let total_time = start_total.elapsed();
    println!("Benchmark complete");
    println!("Total messages: {}", total_messages);
    println!("Header-only resolved recipients: {}", header_only_resolved);
    println!("Messages needing table (headers empty): {}", needed_table);
    println!("Table reads performed: {}", table_forced);
    if total_messages > 0 {
        println!("Header parse avg: {:.2} µs", timing_header_us as f64 / total_messages as f64);
        if table_forced > 0 { println!("Table read avg (only when done): {:.2} µs", timing_table_us as f64 / table_forced as f64); }
        println!("Body extract avg: {:.2} µs", timing_body_us as f64 / total_messages as f64);
    }
    println!("Total elapsed: {:.2?}", total_time);
    if args.csv {
        println!("metric,value");
        println!("total_messages,{}", total_messages);
        println!("header_only_resolved,{}", header_only_resolved);
        println!("needed_table,{}", needed_table);
        println!("table_reads,{}", table_forced);
        println!("header_parse_avg_us,{:.2}", if total_messages>0 { timing_header_us as f64 / total_messages as f64 } else {0.0});
        println!("table_read_avg_us,{:.2}", if table_forced>0 { timing_table_us as f64 / table_forced as f64 } else {0.0});
        println!("body_extract_avg_us,{:.2}", if total_messages>0 { timing_body_us as f64 / total_messages as f64 } else {0.0});
        println!("elapsed_s,{:.3}", total_time.as_secs_f64());
    }
    Ok(())
}
