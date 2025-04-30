use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use tera::{Tera, Context};
use windows::Win32::System::SystemInformation::{GetTickCount64, GlobalMemoryStatusEx, MEMORYSTATUSEX};
use windows::Win32::System::Registry::{RegGetValueW, HKEY_LOCAL_MACHINE, RRF_RT_REG_DWORD};
use windows::core::PCWSTR;
use std::mem::size_of;
use std::ffi::c_void;
use std::ffi::OsStr;
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

async fn index(tmpl: web::Data<Tera>) -> impl Responder {
    let mut context = Context::new();
    context.insert("uptime", &get_uptime());
    context.insert("memory", &get_memory_info());
    context.insert("cpu_speed", &get_cpu_speed());
    context.insert("disk_space", &get_disk_space());

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
