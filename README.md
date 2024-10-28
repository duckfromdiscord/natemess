# natemess

natemess packages the setup needed to communicate with your browser extension over [Native messaging](https://developer.mozilla.org/en-US/docs/Mozilla/Add-ons/WebExtensions/Native_messaging) into a simple async (with `tokio`) Rust library crate. The `natemess::install` module allows you to create the Windows registry keys required to let Firefox know your program exists, and `natemess::io` will communicate with Firefox over STDIO as long as your program does not output anything else to STDOUT when called by the browser.

Some of the stdio handling code came from [guest271314/NativeMessagingHosts](https://github.com/guest271314/NativeMessagingHosts/blob/main/nm_rust.rs). Everything had to be adapted to async Rust.