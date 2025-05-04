use actix_web::HttpRequest;
use actix_web::{web, App, HttpServer, HttpResponse, Result as ActixResult};
use tera::{Tera, Context};
use windows::Win32::System::SystemInformation::{GetTickCount64, GlobalMemoryStatusEx, MEMORYSTATUSEX};
use windows::Win32::System::Registry::{RegGetValueW, HKEY_LOCAL_MACHINE, RRF_RT_REG_DWORD};
use windows::Win32::NetworkManagement::IpHelper::{GetIfTable, MIB_IFTABLE, MIB_IFROW};
use windows::Win32::Foundation::BOOL;
use windows::core::PCWSTR;
use std::ffi::c_void;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;




pub fn get_interface_list() -> Vec<(u32, String)> {
    unsafe {
        let mut size: u32 = 0;

        // First call with None to get the required size
        let result = GetIfTable::<BOOL>(None, &mut size, BOOL(0));
        if result != 122 {  // 122 = ERROR_INSUFFICIENT_BUFFER
            eprintln!("GetIfTable failed to determine buffer size. Error code: {}", result);
            return vec![];
        }

        // Allocate a buffer large enough to hold the table
        let mut buffer = vec![0u8; size as usize];
        let table_ptr = buffer.as_mut_ptr() as *mut MIB_IFTABLE;

        // Second call with allocated buffer
        let result = GetIfTable::<BOOL>(Some(table_ptr), &mut size, BOOL(0));
        if result != 0 {
            eprintln!("GetIfTable failed to retrieve the table. Error code: {}", result);
            return vec![];
        }

        let num_entries = (*table_ptr).dwNumEntries as usize;
        let row_ptr = &(*table_ptr).table as *const MIB_IFROW;

        let mut interfaces = Vec::new();
        for i in 0..num_entries {
            let row = &*row_ptr.add(i);
            let name = String::from_utf8_lossy(&row.bDescr)
                .trim_end_matches(char::from(0))
                .to_string();

            println!("Interface found: index={} name={}", row.dwIndex, name);
            interfaces.push((row.dwIndex, name));
        }

        interfaces
    }
}


fn get_uptime() -> String {
    let uptime_ms = unsafe { GetTickCount64() };
    let seconds = uptime_ms / 1000;
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    format!("{:02}h:{:02}m", hours, minutes)
}

fn get_memory_info() -> String {
    let mut mem_status = MEMORYSTATUSEX {
        dwLength: size_of::<MEMORYSTATUSEX>() as u32,
        ..Default::default()
    };
    unsafe {
        if GlobalMemoryStatusEx(&mut mem_status).is_ok() {
            let total = mem_status.ullTotalPhys / 1024 / 1024;
            let avail = mem_status.ullAvailPhys / 1024 / 1024;
            let used = total - avail;
            return format!("{} MB used / {} MB total", used, total);
        }
    }
    "Unavailable".to_string()
}

fn get_cpu_speed() -> String {
    let subkey = OsStr::new("HARDWARE\\DESCRIPTION\\System\\CentralProcessor\\0")
        .encode_wide().chain(Some(0)).collect::<Vec<_>>();
    let value = OsStr::new("~MHz").encode_wide().chain(Some(0)).collect::<Vec<_>>();

    let mut data: u32 = 0;
    let mut size = std::mem::size_of::<u32>() as u32;

    unsafe {
        if RegGetValueW(
            HKEY_LOCAL_MACHINE,
            PCWSTR(subkey.as_ptr()),
            PCWSTR(value.as_ptr()),
            RRF_RT_REG_DWORD,
            None,
            Some(&mut data as *mut _ as *mut c_void),
            Some(&mut size),
        )
        .is_ok()
        {
            return format!("{} MHz", data);
        }
    }
    "Unavailable".to_string()
}

fn get_disk_space() -> String {
    // Placeholder value — Windows disk APIs are best used with `windows` bindings or sysinfo.
    // You can use `sysinfo` crate if needed.
    "Example: 100 GB used / 500 GB total".to_string()
}

// Index () route handler
async fn index(req: HttpRequest, tmpl: web::Data<tera::Tera>) -> ActixResult<HttpResponse> {
    // Parse query string for selected interface index (from ?iface=...)
    //let query = req.query_string();
    //let params: HashMap<_, _> = form_urlencoded::parse(query.as_bytes()).into_owned().collect();

    // Use the real interface list
    //let interfaces = get_interface_list(); // Vec<(u32, String)>

    //let iface_index = params
    //.get("iface")
    //.and_then(|v| v.parse::<usize>().ok())
    //.filter(|&i| i < interfaces.len())  // ensures the index is valid
    //.unwrap_or(0); // fallback to 0 if not

    // Prepare context for template rendering
    let mut ctx = Context::new();
    ctx.insert("uptime", &get_uptime());
    ctx.insert("memory", &get_memory_info());
    ctx.insert("cpu_speed", &get_cpu_speed());
    ctx.insert("disk_space", &get_disk_space());
    ctx.insert("network_info", &get_interface_list());

    // Render HTML
    let rendered = tmpl.render("index.html", &ctx)
        .map_err(actix_web::error::ErrorInternalServerError)?;
    
    Ok(HttpResponse::Ok().content_type("text/html").body(rendered))
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let tera = Tera::new("templates/**/*").unwrap();
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(tera.clone()))
            .route("/", web::get().to(index))
    })
    .bind("127.0.0.1:5000")?
    .run()
    .await
}
