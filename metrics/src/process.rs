// use errors::otel::OtelError;
// use opentelemetry::{global, metrics::Meter};
// use procfs::process::Process;
// use std::process;
// use tracing::error;

// pub struct ProcessMetricsCollector {
//     meter: Meter,
// }

// impl ProcessMetricsCollector {
//     pub fn new() -> ProcessMetricsCollector {
//         let meter = global::meter("process");

//         return ProcessMetricsCollector { meter };
//     }

//     pub fn collect(&self) -> Result<(), OtelError> {
//         let process_open = self
//             .meter
//             .u64_observable_gauge("process_open_fds")
//             .with_description("Number of open file descriptors.")
//             .init();

//         let v_mem_bytes = self
//             .meter
//             .u64_observable_gauge("process_virtual_memory_bytes")
//             .with_description("Virtual memory size in bytes.")
//             .init();

//         let rss_mem_bytes = self
//             .meter
//             .u64_observable_gauge("process_resident_memory_bytes")
//             .with_description("Resident memory size in bytes.")
//             .init();

//         let cpu_total = self
//             .meter
//             .u64_observable_gauge("process_cpu_seconds_total")
//             .with_description("Total user and system CPU time spent in seconds.")
//             .init();

//         let threads = self
//             .meter
//             .u64_observable_gauge("process_threads")
//             .with_description("Number of OS threads in the process.")
//             .init();

//         let pid = process::id() as i32;
//         let clk_tck: i64 = unsafe { libc::sysconf(libc::_SC_CLK_TCK) }.into();
//         let pagesize: i64 = unsafe { libc::sysconf(libc::_SC_PAGESIZE) }.into();

//         let p = match Process::new(pid) {
//             Ok(p) => Ok(p),
//             Err(e) => {
//                 error!(error = e.to_string(), "error to create the proc struct");
//                 Err(OtelError::ProcFileError {})
//             }
//         }?;

//         self.meter
//             .register_callback(move |cx| {
//                 if let Ok(fd_count) = p.fd_count() {
//                     process_open.observe(cx, fd_count as u64, &[]);
//                 }
//                 if let Ok(limits) = p.limits() {
//                     if let procfs::process::LimitValue::Value(max) =
//                         limits.max_open_files.soft_limit
//                     {
//                         process_open.observe(cx, max, &[]);
//                     }
//                 }

//                 if let Ok(stat) = p.stat() {
//                     v_mem_bytes.observe(cx, stat.vsize, &[]);

//                     let process_resident_memory_bytes = (stat.rss as i64) * pagesize;
//                     rss_mem_bytes.observe(cx, process_resident_memory_bytes as u64, &[]);

//                     let total = (stat.utime + stat.stime) / clk_tck as u64;
//                     cpu_total.observe(cx, total, &[]);
//                     // let past = self.cpu_total.get();
//                     // self.cpu_total.inc_by(total.saturating_sub(past));

//                     threads.observe(cx, stat.num_threads as u64, &[]);
//                 }
//             })
//             .map_err(|e| {
//                 error!(error = e.to_string(), "error to register collect calback");
//                 OtelError::MetricCallbackError {}
//             })?;

//         Ok(())
//     }
// }
