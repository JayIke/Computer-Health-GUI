use actix_web::HttpRequest;
use actix_web::{web, App, HttpServer, HttpResponse, Result as ActixResult};
use tera::{Tera, Context};
use windows::Win32::System::SystemInformation::{GetTickCount64, GlobalMemoryStatusEx, MEMORYSTATUSEX};
use windows::Win32::System::Registry::{RegGetValueW, HKEY_LOCAL_MACHINE, RRF_RT_REG_DWORD};
use windows::Win32::NetworkManagement::IpHelper::{GetIfTable, MIB_IFTABLE};
use windows::Win32::Foundation::BOOL;
use windows::core::PCWSTR;
use std::mem::size_of;
use std::ffi::c_void;
use std::ffi::OsStr;
use std::os::windows::ffi::OsStrExt;
use std::collections::HashMap;
use url::form_urlencoded;


  

fn get_interface_list() -> Vec<(u32, String)> {
    unsafe {
        let mut size = 0;

        // First call to get the required buffer size
        if GetIfTable::<BOOL>(None, &mut size, false.into()) != 0 {
            eprintln!("Failed to get size for interface table.");
            return vec![];
        }

        let mut buffer = vec![0u8; size as usize];
        let table_ptr = buffer.as_mut_ptr() as *mut MIB_IFTABLE;

        if GetIfTable::<BOOL>(Some(table_ptr), &mut size, false.into()) == 0 {
            let table: &MIB_IFTABLE = &*table_ptr;
            let mut interfaces = Vec::new();

            for i in 0..table.dwNumEntries as usize {
                let row = &table.table[i];
                let name = String::from_utf8_lossy(&row.bDescr)
                    .trim_matches(char::from(0))
                    .to_string();

                // Log the interface for debugging
                println!("Interface found: index={} name={}", row.dwIndex, name);

                interfaces.push((row.dwIndex, name));
            }

            interfaces
        } else {
            eprintln!("Failed to get interface table.");
            vec![]
        }
    }
}



fn get_network_stats_for(index: u32) -> String {
    unsafe {
        let mut size = 0;
        // Explicitly specify the type for P0
        let _ = GetIfTable::<BOOL>(None, &mut size, false.into()); // 1st call to get size
        let mut buffer = vec![0u8; size as usize];
        let table_ptr = buffer.as_mut_ptr() as *mut MIB_IFTABLE;

        if GetIfTable::<BOOL>(Some(table_ptr), &mut size, false.into()) == 0 {
            let table = &*table_ptr;
            let num_entries = table.dwNumEntries as usize;

            // Check if the index is within bounds before proceeding
            if index as usize >= num_entries {
                return "Invalid interface index.".to_string();
            }

            // Loop through interfaces and find the correct one
            for i in 0..num_entries {
                let row = &table.table[i];
                if row.dwIndex == index {
                    return format!("{} bytes in / {} bytes out", row.dwInOctets, row.dwOutOctets);
                }
            }
        }
    }
    "Interface not found.".to_string()
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
    let interfaces = get_interface_list(); // Vec<(u32, String)>

    //let iface_index = params
    //.get("iface")
    //.and_then(|v| v.parse::<usize>().ok())
    //.filter(|&i| i < interfaces.len())  // ensures the index is valid
    //.unwrap_or(0); // fallback to 0 if not
    
    let network_info: Vec<(String, String)> = interfaces
    .iter()
    .map(|(idx, name)| {
        let stats = get_network_stats_for(*idx);
        (name.clone(), stats)
    })
    .collect();

    


    // Prepare context for template rendering
    let mut ctx = Context::new();
    ctx.insert("uptime", &get_uptime());
    ctx.insert("memory", &get_memory_info());
    ctx.insert("cpu_speed", &get_cpu_speed());
    ctx.insert("disk_space", &get_disk_space());
    ctx.insert("network_info", &network_info);

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
