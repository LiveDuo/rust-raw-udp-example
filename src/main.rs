
use std::os::raw::{c_int, c_uchar, c_ushort, c_uint};
use std::mem::size_of;
use std::ffi::c_void;

const AF_INET: c_int = 2;
const SOCK_DGRAM: c_int = 2;
const INADDR_LOOPBACK: u32 = 0x7f000001; // 127.0.0.1

const SERVER_PORT: u16 = 8080;

#[repr(C)]
struct SockAddr { sin_len: c_uchar, sin_family: c_uchar, sin_port: c_ushort, sin_addr: c_uint, sin_zero: [u8; 8], }

unsafe extern "C" {
    fn socket(domain: c_int, typ: c_int, protocol: c_int) -> c_int;
    fn bind(sockfd: c_int, addr: *const c_void, addrlen: u32) -> c_int;
    fn recvfrom(sockfd: c_int, buf: *mut c_void, len: usize, flags: c_int, src: *mut c_void, addrlen: *mut u32) -> isize;
    fn sendto(sockfd: c_int, buf: *const c_void, len: usize, flags: c_int, addr: *const c_void, addrlen: u32) -> isize;
    fn close(fd: c_int) -> c_int;
}

// echo "ping" | nc -u 127.0.0.1 8080
fn main() {

    let addr = SockAddr {
        sin_len: size_of::<SockAddr>() as c_uchar,
        sin_family: AF_INET as c_uchar,
        sin_port: SERVER_PORT.to_be(),
        sin_addr: INADDR_LOOPBACK.to_be(),
        sin_zero: [0; 8],
    };

    let sock = unsafe { socket(AF_INET, SOCK_DGRAM, 0) };
    let addrlen = size_of::<SockAddr>() as u32;
    if unsafe { bind(sock, &addr as *const _ as *const c_void, addrlen) } != 0 {
        eprintln!("bind() failed");
        let _ = unsafe { close(sock) };
        return;
    }
    println!("UDP server binded port {}", SERVER_PORT);

    let mut buf = [0u8; 1500];
    loop {
        let mut src = SockAddr { sin_len: 0, sin_family: 0, sin_port: 0, sin_addr: 0, sin_zero: [0; 8], };
        let mut src_len: u32 = addrlen;
        let n = unsafe { recvfrom(sock, buf.as_mut_ptr() as *mut c_void, buf.len(), 0, &mut src as *mut _ as *mut c_void, &mut src_len as *mut u32) };
        if n <= 0 {
            continue;
        }

        let [o0, o1, o2, o3] = u32::from_be(src.sin_addr).to_be_bytes();
        println!("recv {} bytes from {}.{}.{}.{}:{}", n, o0, o1, o2, o3, u16::from_be(src.sin_port));

        let reply = b"Hello from raw UDP server (macOS)\n";
        let sent = unsafe { sendto(sock, reply.as_ptr() as *const c_void, reply.len(), 0, &src as *const _ as *const c_void, src_len) };
        if sent < 0 {
            eprintln!("sendto failed");
        }
    }

}
