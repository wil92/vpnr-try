pub fn code(msg: &[u8; 512], msg_len: usize, addr: u32, port: u16) -> ([u8; 1024], usize) {
    let mut buf: [u8; 1024] = [0; 1024];

    for i in 0..msg_len {
        buf[9 + i] = msg[i];
    }
    let buf_len = msg_len + 9;

    get_buf_len(&mut buf, buf_len);
    get_addr(&mut buf, addr);
    get_port(&mut buf, port);

    (buf, buf_len)
}

pub fn get_buf_len(buf: &mut [u8; 1024], buf_len: usize) {
    buf[0] = ((buf_len >> 8) & 255) as u8;
    buf[1] = (buf_len & 255) as u8;
}

pub fn get_addr(buf: &mut [u8; 1024], addr: u32) {
    buf[2] = ((addr >> 24) & 255) as u8;
    buf[3] = ((addr >> 16) & 255) as u8;
    buf[4] = ((addr >> 8) & 255) as u8;
    buf[5] = (addr & 255) as u8;
}

pub fn get_port(buf: &mut [u8; 1024], port: u16) {
    buf[6] = ((port >> 8) & 255) as u8;
    buf[7] = (port & 255) as u8;
}

#[cfg(test)]
mod tests {
    use crate::tcp::protocol;

    #[test]
    fn protocol_code_success() {
        let msg = String::from("some");
        let msg_u8 = msg.as_bytes();
        let mut msg_u8_pre: [u8; 512] = [0; 512];
        for (i, _) in msg_u8.iter().enumerate() {
            msg_u8_pre[i] = msg_u8[i];
        }
        let (res, res_len) = protocol::code(&msg_u8_pre, msg.len(), 13, 5443);

        let expected: [u8; 1024] = [0; 1024];
        assert_eq!(res_len, msg.len() + 9);
    }

    #[test]
    fn protocol_buf_len_calculation() {
        let mut buf: [u8; 1024] = [0; 1024];
        protocol::get_buf_len(&mut buf, 255);
        assert_eq!(buf[0], 0);
        assert_eq!(buf[1], 255);

        protocol::get_buf_len(&mut buf, 511);
        assert_eq!(buf[0], 1);
        assert_eq!(buf[1], 255);
    }

    #[test]
    fn protocol_addr_calculation() {
        let mut buf: [u8; 1024] = [0; 1024];
        protocol::get_addr(&mut buf, 4294967295);
        assert_eq!(buf[2], 255);
        assert_eq!(buf[3], 255);
        assert_eq!(buf[4], 255);
        assert_eq!(buf[5], 255);
    }

    #[test]
    fn protocol_port_calculation() {
        let mut buf: [u8; 1024] = [0; 1024];
        protocol::get_port(&mut buf, 65535);
        assert_eq!(buf[6], 255);
        assert_eq!(buf[7], 255);
    }
}
