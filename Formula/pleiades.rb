class Pleiades < Formula
  desc "Provider-agnostic terminal AI assistant"
  homepage "https://github.com/CodWasTaken/Pleiades"
  head "https://github.com/CodWasTaken/Pleiades.git", branch: "master"
  license "MIT"
  version "1.1.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/CodWasTaken/Pleiades/releases/download/v1.1.0/pleiades-macos-arm64.tar.gz"
      sha256 "d6fcdba9fe01dca9c52ed7db5ccb0e391578b21b8d3909fb29d900d04f267a3b"
    else
      url "https://github.com/CodWasTaken/Pleiades/releases/download/v1.1.0/pleiades-macos-amd64.tar.gz"
      sha256 "514517f41dabc0dcf74c0cb02da4f2b20a958a5cc857ee6bdb5a7fc76c306c62"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/CodWasTaken/Pleiades/releases/download/v1.1.0/pleiades-linux-arm64.tar.gz"
      sha256 "b90c5213e62cad5d4c8a76be2b27230059403521687e6a3439bc1098429e64e5"
    else
      url "https://github.com/CodWasTaken/Pleiades/releases/download/v1.1.0/pleiades-linux-amd64.tar.gz"
      sha256 "3231e131fbb8678534045daea9c054e2add5e41380e622e26d291b7dc786836e"
    end
  end

  def install
    bin.install "pleiades"
  end

  test do
    assert_match "Pleiades", shell_output("#{bin}/pleiades --help")
  end
end
