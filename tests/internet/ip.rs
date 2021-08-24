use super::super::*;

mod ip_header {

    use super::*;
    use crate::ip_number::*;
    use std::io::Cursor;

    fn combine_v4(v4: &Ipv4Header, ext: &Ipv4Extensions) -> IpHeader {
        IpHeader::Version4(
            {
                let mut v4 = v4.clone();
                v4.protocol = if ext.auth.is_some() {
                    AUTH
                } else {
                    UDP
                };
                v4.header_checksum = v4.calc_header_checksum().unwrap();
                v4
            },
            ext.clone(),
        )
    }

    fn combine_v6(v6: &Ipv6Header, ext: &Ipv6Extensions) -> IpHeader {
        let (ext, next_header) = {
            let mut ext = ext.clone();
            let next_header = ext.set_next_headers(UDP);
            (ext, next_header)
        };
        IpHeader::Version6(
            {
                let mut v6 = v6.clone();
                v6.next_header = next_header;
                v6
            },
            ext,
        )
    }

    proptest!{
        #[test]
        fn read_from_slice(
            v4 in ipv4_any(),
            v4_exts in ipv4_extensions_any(),
            v6 in ipv6_any(),
            v6_exts in ipv6_extensions_any(),
        ) {
            // v4
            {
                let header = combine_v4(&v4, &v4_exts);
                let mut buffer = Vec::with_capacity(header.header_len() + 1);
                header.write(&mut buffer).unwrap();
                buffer.push(1); // add some value to check the return slice

                // read
                {
                    let actual = IpHeader::read_from_slice(&buffer).unwrap();
                    assert_eq!(actual.0, header);
                    assert_eq!(actual.1, header.next_header().unwrap());
                    assert_eq!(actual.2, &buffer[buffer.len() - 1..]);
                }

                // read error ipv4 header
                IpHeader::read_from_slice(&buffer[..1]).unwrap_err();

                // read error ipv4 extensions
                if v4_exts.header_len() > 0 {
                    IpHeader::read_from_slice(&buffer[..v4.header_len() + 1]).unwrap_err();
                }
            }

            // v6
            {
                let header = combine_v6(&v6, &v6_exts);
                let mut buffer = Vec::with_capacity(header.header_len() + 1);
                header.write(&mut buffer).unwrap();
                buffer.push(1); // add some value to check the return slice

                // read
                {
                    let actual = IpHeader::read_from_slice(&buffer).unwrap();
                    assert_eq!(actual.0, header);
                    assert_eq!(actual.1, header.next_header().unwrap());
                    assert_eq!(actual.2, &buffer[buffer.len() - 1..]);
                }

                // read error header
                IpHeader::read_from_slice(&buffer[..1]).unwrap_err();

                // read error ipv4 extensions
                if v6_exts.header_len() > 0 {
                    IpHeader::read_from_slice(&buffer[..Ipv6Header::SERIALIZED_SIZE + 1]).unwrap_err();
                }
            }
        }
    }

    proptest!{
        #[test]
        fn read(
            v4 in ipv4_any(),
            v4_exts in ipv4_extensions_any(),
            v6 in ipv6_any(),
            v6_exts in ipv6_extensions_any(),
        ) {
            // v4
            {
                let header = combine_v4(&v4, &v4_exts);
                let mut buffer = Vec::with_capacity(header.header_len());
                header.write(&mut buffer).unwrap();

                // read
                {
                    let mut cursor = Cursor::new(&buffer);
                    let actual = IpHeader::read(&mut cursor).unwrap();
                    assert_eq!(actual.0, header);
                    assert_eq!(actual.1, header.next_header().unwrap());
                }

                // read error ipv4 header
                {
                    let mut cursor = Cursor::new(&buffer[..1]);
                    IpHeader::read(&mut cursor).unwrap_err();
                }

                // read error ipv4 extensions
                if v4_exts.header_len() > 0 {
                    let mut cursor = Cursor::new(&buffer[..v4.header_len() + 1]);
                    IpHeader::read(&mut cursor).unwrap_err();
                }
            }

            // v6
            {
                let header = combine_v6(&v6, &v6_exts);
                let mut buffer = Vec::with_capacity(header.header_len());
                header.write(&mut buffer).unwrap();

                // read
                {
                    let mut cursor = Cursor::new(&buffer);
                    let actual = IpHeader::read(&mut cursor).unwrap();
                    assert_eq!(actual.0, header);
                    assert_eq!(actual.1, header.next_header().unwrap());
                }

                // read error header
                {
                    let mut cursor = Cursor::new(&buffer[..1]);
                    IpHeader::read(&mut cursor).unwrap_err();
                }

                // read error ipv4 extensions
                if v6_exts.header_len() > 0 {
                    let mut cursor = Cursor::new(&buffer[..Ipv6Header::SERIALIZED_SIZE + 1]);
                    IpHeader::read(&mut cursor).unwrap_err();
                }
            }
        }
    }

    proptest!{
        #[test]
        fn write(
            v4 in ipv4_any(),
            v4_exts in ipv4_extensions_any(),
            v6 in ipv6_any(),
            v6_exts in ipv6_extensions_any(),
        ) {
            // v4
            {
                let header = combine_v4(&v4, &v4_exts);
                let mut buffer = Vec::with_capacity(header.header_len());
                header.write(&mut buffer).unwrap();

                let actual = IpHeader::read_from_slice(&buffer).unwrap().0;
                assert_eq!(header, actual);

                // write error v4 header
                let mut writer = TestWriter::with_max_size(1);
                assert_eq!(
                    writer.error_kind(),
                    header.write(&mut writer).unwrap_err().io_error().unwrap().kind()
                );

                // write error v6 extension headers
                if v4_exts.header_len() > 0 {
                    let mut writer = TestWriter::with_max_size(v4.header_len() + 1);
                    assert_eq!(
                        writer.error_kind(),
                        header.write(&mut writer).unwrap_err().io_error().unwrap().kind()
                    );
                }
            }

            // v6
            {
                let header = combine_v6(&v6, &v6_exts);

                // normal write
                let mut buffer = Vec::with_capacity(header.header_len());
                header.write(&mut buffer).unwrap();

                let actual = IpHeader::read_from_slice(&buffer).unwrap().0;
                assert_eq!(header, actual);

                // write error v6 header
                {
                    let mut writer = TestWriter::with_max_size(1);
                    assert_eq!(
                        writer.error_kind(),
                        header.write(&mut writer).unwrap_err().io_error().unwrap().kind()
                    );
                }

                // write error v6 extension headers
                if v6_exts.header_len() > 0 {
                    let mut writer = TestWriter::with_max_size(Ipv6Header::SERIALIZED_SIZE + 1);
                    assert_eq!(
                        writer.error_kind(),
                        header.write(&mut writer).unwrap_err().io_error().unwrap().kind()
                    );
                }
            }
        }
    }

    #[test]
    fn header_len() {
        // TODO
    }

    #[test]
    fn next_header() {
        // TODO
    }

    #[test]
    fn debug() {
        // TODO
    }

    #[test]
    fn clone_eq() {
        // TODO
    }

    #[test]
    fn read_ip_header_version_error() {
        use std::io::Cursor;
        let input = Ipv6Header {
            traffic_class: 1,
            flow_label: 0x81806,
            payload_length: 0x8021,
            next_header: 30,
            hop_limit: 40,
            source: [1, 2, 3, 4, 5, 6, 7, 8,
                     9,10,11,12,13,14,15,16],
            destination: [21,22,23,24,25,26,27,28,
                          29,30,31,32,33,34,35,36]
        };
        //serialize
        let mut buffer: Vec<u8> = Vec::with_capacity(20);
        input.write(&mut buffer).unwrap();
        assert_eq!(40, buffer.len());

        //corrupt the version
        buffer[0] = 0xff;

        //deserialize with read
        {
            let mut cursor = Cursor::new(&buffer);
            assert_matches!(IpHeader::read(&mut cursor), Err(ReadError::IpUnsupportedVersion(0xf)));
        }

        //deserialize with read_from_slice
        assert_matches!(
            IpHeader::read_from_slice(&buffer), 
            Err(ReadError::IpUnsupportedVersion(0xf))
        );
        //also check that an error is thrown when the slice is too small 
        //to even read the version
        assert_matches!(
            IpHeader::read_from_slice(&buffer[buffer.len()..]), 
            Err(ReadError::UnexpectedEndOfSlice(1))
        );
    }
} // mod ip_header

mod ip_number {

    #[test]
    fn is_ipv6_ext_header_value() {
        use crate::IpNumber;
        use crate::ip_number::*;
        let ext_ids = [
            IPV6_HOP_BY_HOP,
            IPV6_ROUTE,
            IPV6_FRAG,
            ENCAP_SEC,
            AUTH,
            IPV6_DEST_OPTIONS,
            MOBILITY,
            HIP,
            SHIM6 as u8,
            EXP0 as u8,
            EXP1 as u8
        ];

        for i in 0..std::u8::MAX {
            assert_eq!(
                ext_ids.contains(&i),
                IpNumber::is_ipv6_ext_header_value(i)
            );
        }
    }

    #[test]
    fn ip_number_eq_check() {
        use crate::ip_number::*;
        use crate::IpNumber::*;
        let pairs = &[
            (IPV6_HOP_BY_HOP, IPv6HeaderHopByHop),
            (ICMP, Icmp),
            (IGMP, Igmp),
            (GGP, Ggp),
            (IPV4, IPv4),
            (STREAM, Stream),
            (TCP, Tcp),
            (UDP, Udp),
            (IPV6, Ipv6),
            (IPV6_ROUTE, IPv6RouteHeader),
            (IPV6_FRAG, IPv6FragmentationHeader),
            (ENCAP_SEC, EncapsulatingSecurityPayload),
            (AUTH, AuthenticationHeader),
            (IPV6_DEST_OPTIONS, IPv6DestinationOptions),
            (MOBILITY, MobilityHeader),
            (HIP, Hip),
            (SHIM6, Shim6),
            (EXP0, ExperimentalAndTesting0),
            (EXP1, ExperimentalAndTesting1),
        ];
        for (raw, enum_value) in pairs {
            assert_eq!(*raw, *enum_value as u8);
        }
    }

    #[test]
    fn debug() {
        // TODO
    }

    #[test]
    fn clone_eq() {
        // TODO
    }

} // mod ip_number