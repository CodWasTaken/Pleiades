class Pleiades < Formula
  desc "Provider-agnostic terminal AI assistant"
  homepage "https://github.com/CodWasTaken/Pleiades"
  head "https://github.com/CodWasTaken/Pleiades.git", branch: "master"
  license "MIT"
  version "1.2.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/CodWasTaken/Pleiades/releases/download/v1.2.0/pleiades-macos-arm64.tar.gz"
      sha256 "d4551fbe8942ed1f891e264bc8a4f4e54f25796c015517b4bd523d15c03e3c0c"
    else
      url "https://github.com/CodWasTaken/Pleiades/releases/download/v1.2.0/pleiades-macos-amd64.tar.gz"
      sha256 "e3198c87282858b41496241f6c775f14fbd47c5986e8885449f447788f29c5ba"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/CodWasTaken/Pleiades/releases/download/v1.2.0/pleiades-linux-arm64.tar.gz"
      sha256 "fca242fb48b617c5f7250e95bb74d14a82a4dc8fec9da63c3b12c069e86fb66c"
    else
      url "https://github.com/CodWasTaken/Pleiades/releases/download/v1.2.0/pleiades-linux-amd64.tar.gz"
      sha256 "08cbe9a74284fa12c6ef4af202906e745135ee6abffe8c8e7aff4c90506f2b6b"
    end
  end

  def install
    bin.install "pleiades"
  end

  test do
    assert_match "Pleiades", shell_output("#{bin}/pleiades --help")
  end
end
