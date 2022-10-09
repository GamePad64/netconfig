cfg_if::cfg_if! {
    if #[cfg(target_os = "linux")] {
        nix::ioctl_write_ptr_bad!(siocsifmtu, libc::SIOCSIFMTU, super::ifreq::ifreq);
        nix::ioctl_write_ptr_bad!(siocsifflags, libc::SIOCSIFFLAGS, super::ifreq::ifreq);
        nix::ioctl_write_ptr_bad!(siocsifaddr, libc::SIOCSIFADDR, super::ifreq::ifreq);
        nix::ioctl_write_ptr_bad!(siocsifdstaddr, libc::SIOCSIFDSTADDR, super::ifreq::ifreq);
        nix::ioctl_write_ptr_bad!(siocsifbrdaddr, libc::SIOCSIFBRDADDR, super::ifreq::ifreq);
        nix::ioctl_write_ptr_bad!(siocsifnetmask, libc::SIOCSIFNETMASK, super::ifreq::ifreq);
        nix::ioctl_write_ptr_bad!(siocsifhwaddr, libc::SIOCSIFHWADDR, super::ifreq::ifreq);

        nix::ioctl_read_bad!(siocgifmtu, libc::SIOCGIFMTU, super::ifreq::ifreq);
        nix::ioctl_read_bad!(siocgifflags, libc::SIOCGIFFLAGS, super::ifreq::ifreq);
        nix::ioctl_read_bad!(siocgifaddr, libc::SIOCGIFADDR, super::ifreq::ifreq);
        nix::ioctl_read_bad!(siocgifdstaddr, libc::SIOCGIFDSTADDR, super::ifreq::ifreq);
        nix::ioctl_read_bad!(siocgifbrdaddr, libc::SIOCGIFBRDADDR, super::ifreq::ifreq);
        nix::ioctl_read_bad!(siocgifnetmask, libc::SIOCGIFNETMASK, super::ifreq::ifreq);
        nix::ioctl_read_bad!(siocgifhwaddr, libc::SIOCGIFHWADDR, super::ifreq::ifreq);
    } else if #[cfg(target_os = "macos")] {
        nix::ioctl_readwrite!(siocgifmtu, b'i', 51, super::ifreq::ifreq);
        nix::ioctl_write_ptr!(siocsifmtu, b'i', 52, super::ifreq::ifreq);
        nix::ioctl_readwrite!(siocgifflags, b'i', 17, super::ifreq::ifreq);
        nix::ioctl_write_ptr!(siocsifflags, b'i', 16, super::ifreq::ifreq);
        nix::ioctl_write_ptr!(siocaifaddr4, b'i', 26, super::ifreq::ifaliasreq4);
        nix::ioctl_write_ptr!(siocaifaddr6, b'i', 26, super::ifreq::ifaliasreq6);
        nix::ioctl_write_ptr!(siocdifaddr, b'i', 25, super::ifreq::ifreq);
    }
}
