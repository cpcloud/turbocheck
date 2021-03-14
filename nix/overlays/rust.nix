self: super:
let
  rustChannel = self.rustChannelOf {
    rustToolchain = ../../rust-toolchain;
    sha256 = "19va5fnpbqkllw35rc19q2mixrx9p3m3q5dyi0881c8rcsja7rxc";
    installDoc = false;
  };
in
{
  inherit rustChannel;
  rustc = rustChannel.rust;
  cargo = rustChannel.rust;
}
