# Remote start stop

This tool allows simply starting / stopping remote service through SSH and opening web browser.

## Building

### Windows MSVC

For sure there are many options available, but this worked for me:
1. Download [libsodium-1.0.17-msvc.zip](https://download.libsodium.org/libsodium/releases/libsodium-1.0.17-msvc.zip)
   and [libressl-2.9.2.tar.gz](https://ftp.openbsd.org/pub/OpenBSD/LibreSSL/libressl-2.9.2.tar.gz).
2. Install [CMake](https://cmake.org/download/) if necessary.
3. Build and install libressl:
   * `mkdir build`
   * `cd build`
   * `cmake ..`
   * `cmake --build . --config Release --target install`
4. Use following ENV variables when building:
   * `SODIUM_LIB_DIR=WHERE_YOU_UNPACKED_SODIUM\libsodium-1.0.17-msvc\x64\Release\v141\static`
   * `SODIUM_STATIC=yes`
   * `OPENSSL_DIR=c:\Program Files (x86)\LibreSSL`
