self: super:
let
  channel = self.rustChannelOf {
    rustToolchain = ../../rust-toolchain;
    sha256 = "19va5fnpbqkllw35rc19q2mixrx9p3m3q5dyi0881c8rcsja7rxc";
  };
  rust = channel.rust;
in
{
  rustc = rust;
  cargo = rust;
}
