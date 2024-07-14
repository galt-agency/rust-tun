//            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//                    Version 2, December 2004
//
// Copyleft (ↄ) meh. <meh@schizofreni.co> | http://meh.schizofreni.co
//
// Everyone is permitted to copy and distribute verbatim or modified
// copies of this license document, and changing it is allowed as long
// as the name is changed.
//
//            DO WHAT THE FUCK YOU WANT TO PUBLIC LICENSE
//   TERMS AND CONDITIONS FOR COPYING, DISTRIBUTION AND MODIFICATION
//
//  0. You just DO WHAT THE FUCK YOU WANT TO.
#![allow(unused_variables)]

use std::io::{Read, Write};
use std::net::IpAddr;
use std::os::unix::io::{AsRawFd, IntoRawFd, RawFd};

use crate::configuration::Configuration;
use crate::device::AbstractDevice;
use crate::error::{Error, Result};
use crate::platform::posix::{self, Fd, Tun};

/// A TUN device for Android.
pub struct Device {
    tun: Tun,
    address_: Option<IpAddr>,
    netmask_: Option<IpAddr>,
}

impl AsRef<dyn AbstractDevice + 'static> for Device {
    fn as_ref(&self) -> &(dyn AbstractDevice + 'static) {
        self
    }
}

impl AsMut<dyn AbstractDevice + 'static> for Device {
    fn as_mut(&mut self) -> &mut (dyn AbstractDevice + 'static) {
        self
    }
}

impl Device {
    /// Create a new `Device` for the given `Configuration`.
    pub fn new(config: &Configuration) -> Result<Self> {
        let close_fd_on_drop = config.close_fd_on_drop.unwrap_or(true);
        let fd = match config.raw_fd {
            Some(raw_fd) => raw_fd,
            _ => return Err(Error::InvalidConfig),
        };
        let device = {
            let mtu = config.mtu.unwrap_or(crate::DEFAULT_MTU);
            let tun = Fd::new(fd, close_fd_on_drop).map_err(|_| std::io::Error::last_os_error())?;

            Device {
                tun: Tun::new(tun, mtu, false),
                address_: config.address,
                netmask_: config.netmask,
            }
        };

        Ok(device)
    }

    /// Split the interface into a `Reader` and `Writer`.
    pub fn split(self) -> (posix::Reader, posix::Writer) {
        (self.tun.reader, self.tun.writer)
    }

    /// Set non-blocking mode
    pub fn set_nonblock(&self) -> std::io::Result<()> {
        self.tun.set_nonblock()
    }
}

impl Read for Device {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.tun.read(buf)
    }
}

impl Write for Device {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.tun.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.tun.flush()
    }
}

impl AbstractDevice for Device {
    fn tun_name(&self) -> Result<String> {
        Ok("".to_string())
    }

    fn set_tun_name(&mut self, value: &str) -> Result<()> {
        Err(Error::NotImplemented)
    }

    fn enabled(&mut self, value: bool) -> Result<()> {
        Ok(())
    }

    fn address(&self) -> Result<IpAddr> {
        if let Some(value) = self.address_ {
            Ok(value)
        } else {
            Err(Error::NotImplemented)
        }
    }

    fn set_address(&mut self, value: IpAddr) -> Result<()> {
        self.address_ = Some(value);
        Ok(())
    }

    fn destination(&self) -> Result<IpAddr> {
        Err(Error::NotImplemented)
    }

    fn set_destination(&mut self, _value: IpAddr) -> Result<()> {
        Ok(())
    }

    fn broadcast(&self) -> Result<IpAddr> {
        Err(Error::NotImplemented)
    }

    fn set_broadcast(&mut self, _value: IpAddr) -> Result<()> {
        Ok(())
    }

    fn netmask(&self) -> Result<IpAddr> {
        if let Some(value) = self.netmask_ {
            Ok(value)
        } else {
            Err(Error::NotImplemented)
        }
    }

    fn set_netmask(&mut self, value: IpAddr) -> Result<()> {
        self.netmask_ = Some(value);
        Ok(())
    }

    fn mtu(&self) -> Result<u16> {
        // TODO: must get the mtu from the underlying device driver
        Ok(self.tun.mtu())
    }

    fn set_mtu(&mut self, value: u16) -> Result<()> {
        // TODO: must set the mtu to the underlying device driver
        self.tun.set_mtu(value);
        Ok(())
    }

    fn packet_information(&self) -> bool {
        self.tun.packet_information()
    }
}

impl AsRawFd for Device {
    fn as_raw_fd(&self) -> RawFd {
        self.tun.as_raw_fd()
    }
}

impl IntoRawFd for Device {
    fn into_raw_fd(self) -> RawFd {
        self.tun.into_raw_fd()
    }
}
