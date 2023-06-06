pub fn code_string(
    data: &[u8],
    data_len: usize,
    id_connection: u16,
    flags: u8,
    addr: u32,
    port: u16,
) -> Vec<Vec<u8>> {
    let mut res: Vec<Vec<u8>> = Vec::new();
    let mut data_vec = Vec::new();
    for i in 0..data_len {
        data_vec.push(data[i]);
    }
    let mut bytes = data_vec.into_iter();
    let mut byt = bytes.next();
    loop {
        if byt == None {
            break;
        }
        let mut msg_byte: [u8; 200] = [0; 200];
        let mut msg_len = 0;
        while msg_len < 200 && byt != None {
            msg_byte[msg_len] = byt.unwrap();
            msg_len += 1;
            byt = bytes.next();
        }
        res.push(code_block(
            &msg_byte,
            msg_len,
            id_connection,
            flags,
            addr,
            port,
        ));
    }
    res
}

pub fn decode_string(data: &[u8], data_len: usize) -> (Vec<(Vec<u8>, u16, u8, u32, u16)>, usize) {
    let mut res = Vec::new();
    let mut shift = 0;
    loop {
        if shift >= data_len {
            break;
        }
        let size = data[shift] as usize;
        if size > 0 && data_len >= size + shift {
            let mut id = (data[shift + 1] as u16) << 8;
            id += data[shift + 2] as u16;

            let flags = data[shift + 3] as u8;

            let mut addr = (data[shift + 4] as u32) << 24;
            addr += (data[shift + 5] as u32) << 16;
            addr += (data[shift + 6] as u32) << 8;
            addr += data[shift + 7] as u32;

            let mut port = (data[shift + 8] as u16) << 8;
            port += data[shift + 9] as u16;

            let mut msg = Vec::new();

            for i in 0..(size - 10) {
                msg.push(data[shift + i + 10]);
            }

            shift += size;

            res.push((msg, id, flags, addr, port));
        } else {
            break;
        }
    }
    (res, shift)
}

pub fn code_block(
    msg: &[u8],
    msg_len: usize,
    msg_id: u16,
    flags: u8,
    addr: u32,
    port: u16,
) -> Vec<u8> {
    let mut buf = Vec::new();

    let buf_len = msg_len + 10;
    get_buf_len(&mut buf, buf_len);
    get_msg_id(&mut buf, msg_id);
    buf.push(flags);
    get_addr(&mut buf, addr);
    get_port(&mut buf, port);

    for i in 0..msg_len {
        buf.push(msg[i]);
    }

    buf
}

pub fn get_buf_len(buf: &mut Vec<u8>, buf_len: usize) {
    buf.push((buf_len & 255) as u8);
}

pub fn get_msg_id(buf: &mut Vec<u8>, msg_id: u16) {
    buf.push(((msg_id >> 8) & 255) as u8);
    buf.push((msg_id & 255) as u8);
}

pub fn get_addr(buf: &mut Vec<u8>, addr: u32) {
    buf.push(((addr >> 24) & 255) as u8);
    buf.push(((addr >> 16) & 255) as u8);
    buf.push(((addr >> 8) & 255) as u8);
    buf.push((addr & 255) as u8);
}

pub fn get_port(buf: &mut Vec<u8>, port: u16) {
    buf.push(((port >> 8) & 255) as u8);
    buf.push((port & 255) as u8);
}

#[cfg(test)]
mod tests {
    use crate::tcp::protocol;

    #[test]
    fn protocol_decode_string_test() {
        let data = String::from("asdf asdf asdfasdf asdf");
        let res = protocol::code_string(data.as_bytes(), data.len(), 1, 3, 2383468972, 0);
        let code_data = &res[0];
        let mut data2 = [0; 200];
        for i in 0..code_data.len() {
            data2[i] = code_data[i];
            data2[i + code_data.len()] = code_data[i];
            data2[i + code_data.len() * 2] = code_data[i];
        }
        let (res, shift) = protocol::decode_string(&data2, code_data.len() * 2 + 10);
        assert_eq!(2, res.len());
        assert_eq!(code_data.len() * 2, shift);
        let origin_data = data.as_bytes();

        // validate id
        assert_eq!(1, res[0].1);
        // validate flags
        assert_eq!(3, res[0].2);
        // validate addr
        assert_eq!(res[0].3, 2383468972);
        // validate port
        assert_eq!(res[0].4, 0);

        assert_eq!(data.len(), res[0].0.len());
        for i in 0..res[0].0.len() {
            assert_eq!(origin_data[i], res[0].0[i]);
        }
    }

    #[test]
    fn protocol_code_string_test_codification() {
        let data = String::from("asdf asdf asdfasdf asdf");
        let res = protocol::code_string(data.as_bytes(), data.len(), 1, 1, 2383468972, 80);
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].len(), data.len() + 10);

        // validate id
        assert_eq!(res[0][2], 1);
        // validate flags
        assert_eq!(res[0][3], 1);
        // validate addr
        assert_eq!(res[0][4], 142);
        assert_eq!(res[0][5], 16);
        assert_eq!(res[0][6], 217);
        assert_eq!(res[0][7], 172);
        // validate port
        assert_eq!(res[0][8], 0);
        assert_eq!(res[0][9], 80);

        let data_bytes = data.as_bytes();
        for i in 0..data.len() {
            assert_eq!(data_bytes[i], res[0][i + 10]);
        }
    }

    #[test]
    fn protocol_code_string_test_chunks() {
        let mut data = String::from("");
        for _ in 0..2150 {
            data.push_str("w");
        }
        let res = protocol::code_string(data.as_bytes(), data.len(), 1, 1, 0, 0);
        assert_eq!(res.len(), 11);
    }

    #[test]
    fn protocol_code_block_success() {
        let msg = String::from("some");
        let msg_u8 = msg.as_bytes();
        let mut msg_u8_pre: [u8; 200] = [0; 200];
        for (i, _) in msg_u8.iter().enumerate() {
            msg_u8_pre[i] = msg_u8[i];
        }
        let res = protocol::code_block(&msg_u8_pre, msg.len(), 432, 2, 13, 5443);

        let expected: &[u8] = "some".as_bytes();
        assert_eq!(res.len(), msg.len() + 10);
        for i in 0..4 {
            assert_eq!(res[i + 10], expected[i]);
        }

        // validate msg id
        assert_eq!(res[1], 1);
        assert_eq!(res[2], 176);

        // validate flags
        assert_eq!(res[3], 2);

        // validate addr
        assert_eq!(res[4], 0);
        assert_eq!(res[5], 0);
        assert_eq!(res[6], 0);
        assert_eq!(res[7], 13);

        // validate port
        assert_eq!(res[8], 21);
        assert_eq!(res[9], 67);
    }

    #[test]
    fn protocol_code_block_with_empty_msg_success() {
        let res = protocol::code_block(b"", 0, 432, 2, 13, 5443);

        // validate size
        assert_eq!(res[0], 10);

        // validate msg id
        assert_eq!(res[1], 1);
        assert_eq!(res[2], 176);

        // validate flags
        assert_eq!(res[3], 2);

        // validate addr
        assert_eq!(res[4], 0);
        assert_eq!(res[5], 0);
        assert_eq!(res[6], 0);
        assert_eq!(res[7], 13);

        // validate port
        assert_eq!(res[8], 21);
        assert_eq!(res[9], 67);
    }

    #[test]
    fn protocol_buf_len_calculation() {
        let mut buf = Vec::new();
        protocol::get_buf_len(&mut buf, 255);
        assert_eq!(buf[0], 255);
    }

    #[test]
    fn protocol_msg_id_calculation() {
        let mut buf = Vec::new();
        protocol::get_msg_id(&mut buf, 1279);
        assert_eq!(buf[0], 4);
        assert_eq!(buf[1], 255);
    }

    #[test]
    fn protocol_addr_calculation() {
        let mut buf = Vec::new();
        protocol::get_addr(&mut buf, 4294967295);
        assert_eq!(buf[0], 255);
        assert_eq!(buf[1], 255);
        assert_eq!(buf[2], 255);
        assert_eq!(buf[3], 255);
    }

    #[test]
    fn protocol_port_calculation() {
        let mut buf = Vec::new();
        protocol::get_port(&mut buf, 65535);
        assert_eq!(buf[0], 255);
        assert_eq!(buf[1], 255);
    }
}
