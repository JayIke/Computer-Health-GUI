use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use tera::{Tera, Context};
use windows::Win32::System::SystemInformation::{GetTickCount64, GlobalMemoryStatusEx, MEMORYSTATUSEX};
use windows::Win32::System::Registry::{RegGetValueW, HKEY_LOCAL_MACHINE, RRF_RT_REG_DWORD, PCWSTR};
use windows::Win32::NetworkManagement::IpHelper::{GetIfTable, MIB_IFTABLE, MIB_IFROW};
use std::mem::{size_of, zeroed};
use std::ptr::null_mut;
use std::ffi::{c_void, OsStr};
use std::os::windows::ffi::OsStrExt;

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
        if GlobalMemoryStatusEx(&mut mem_status).as_bool() {
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
    let mut size = size_of::<u32>() as u32;

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

fn get_network_stats() -> String {
    unsafe {
        let mut buffer_len: u32 = 0;
        let _ = GetIfTable(null_mut(), &mut buffer_len, false.into());
        let mut buffer = vec![0u8; buffer_len as usize];
        let if_table: *mut MIB_IFTABLE = buffer.as_mut_ptr() as *mut MIB_IFTABLE;

        if GetIfTable(if_table, &mut buffer_len, false.into()).is_ok() {
            let table = &*if_table;
            if table.dwNumEntries > 0 {
                let row: MIB_IFROW = table.table[0];
                return format!("{} bytes in / {} bytes out", row.dwInOctets, row.dwOutOctets);
            }
        }
    }
    "Unavailable".to_string()
}

fn get_disk_space() -> String {
    "Example: 100 GB used / 500 GB total".to_string()
}

async fn index(tmpl: web::Data<Tera>) -> impl Responder {
    let mut context = Context::new();
    context.insert("uptime", &get_uptime());
    context.insert("memory", &get_memory_info());
    context.insert("cpu_speed", &get_cpu_speed());
    context.insert("disk_space", &get_disk_space());
    context.insert("network_stats", &get_network_stats());

    let rendered = tmpl.render("index.html", &context).unwrap();
    HttpResponse::Ok().content_type("text/html").body(rendered)
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
