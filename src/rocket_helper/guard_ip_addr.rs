use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use rocket::Outcome;
use rocket::request::{self, Request, FromRequest};

/// The request guard used for getting an IP address from a client.
#[derive(Debug, Clone)]
pub struct ClientRealAddr {
    /// IP address from a client.
    pub ip: IpAddr
}

pub fn get_request_client_ip(request: &Request) -> Option<ClientRealAddr> {
    match request.client_ip() {
        Some(ip) => Some(ClientRealAddr {
            ip
        }),
        None => {
            let forwarded_for_ip: Option<&str> = request.headers().get("x-forwarded-for").next(); // Only fetch the first one.

            match forwarded_for_ip {
                Some(forwarded_for_ip) => {
                    let forwarded_for_ip = forwarded_for_ip.split(",").next(); // Only fetch the first one.

                    match forwarded_for_ip {
                        Some(forwarded_for_ip) => match forwarded_for_ip.trim().parse::<IpAddr>() {
                            Ok(ip) => Some(ClientRealAddr {
                                ip
                            }),
                            Err(_) => match request.remote() {
                                Some(addr) => Some(ClientRealAddr {
                                    ip: addr.ip()
                                }),
                                None => None
                            }
                        },
                        None => match request.remote() {
                            Some(addr) => Some(ClientRealAddr {
                                ip: addr.ip()
                            }),
                            None => None
                        }
                    }
                }
                None => match request.remote() {
                    Some(addr) => Some(ClientRealAddr {
                        ip: addr.ip()
                    }),
                    None => None
                }
            }
        }
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for ClientRealAddr {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        match get_request_client_ip(request) {
            Some(client_addr) => Outcome::Success(client_addr),
            None => Outcome::Forward(())
        }
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for &'a ClientRealAddr {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let cache: &Option<ClientRealAddr> = request.local_cache(|| get_request_client_ip(request));

        match &cache {
            Some(client_addr) => Outcome::Success(client_addr),
            None => Outcome::Forward(())
        }
    }
}

impl ClientRealAddr {
    /// Get an `Ipv4Addr` instance.
    pub fn get_ipv4(&self) -> Option<Ipv4Addr> {
        match &self.ip {
            IpAddr::V4(ipv4) => {
                Some(ipv4.clone())
            }
            IpAddr::V6(ipv6) => {
                ipv6.to_ipv4()
            }
        }
    }

    /// Get an IPv4 string.
    pub fn get_ipv4_string(&self) -> Option<String> {
        match &self.ip {
            IpAddr::V4(ipv4) => {
                Some(ipv4.to_string())
            }
            IpAddr::V6(ipv6) => {
                ipv6.to_ipv4().map(|ipv6| ipv6.to_string())
            }
        }
    }

    /// Get an `Ipv6Addr` instance.
    pub fn get_ipv6(&self) -> Ipv6Addr {
        match &self.ip {
            IpAddr::V4(ipv4) => {
                ipv4.to_ipv6_mapped()
            }
            IpAddr::V6(ipv6) => {
                ipv6.clone()
            }
        }
    }

    /// Get an IPv6 string.
    pub fn get_ipv6_string(&self) -> String {
        match &self.ip {
            IpAddr::V4(ipv4) => {
                ipv4.to_ipv6_mapped().to_string()
            }
            IpAddr::V6(ipv6) => {
                ipv6.to_string()
            }
        }
    }
}