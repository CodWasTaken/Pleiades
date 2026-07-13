class Pleiades < Formula
  desc "Provider-agnostic terminal AI assistant"
  homepage "https://github.com/CodWasTaken/Pleiades"
  head "https://github.com/CodWasTaken/Pleiades.git", branch: "master"
  license "MIT"
  version "1.0.1"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/CodWasTaken/Pleiades/releases/download/v1.0.1/pleiades-macos-arm64.tar.gz"
      sha256 "a1377567c8d767f0e72b3a563b84cfb9355433dc29ee8d0b358d329c504e0740"
    else
      url "https://github.com/CodWasTaken/Pleiades/releases/download/v1.0.1/pleiades-macos-amd64.tar.gz"
      sha256 "605e86486f954147e9778778d65dd99aeab4c0a29719d346959dd3e79d628427"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/CodWasTaken/Pleiades/releases/download/v1.0.1/pleiades-linux-arm64.tar.gz"
      sha256 "f9e41ddbb980a906483f064173e73ca50130a8b0770e9b3a6a32c782e10c4e0c"
    else
      url "https://github.com/CodWasTaken/Pleiades/releases/download/v1.0.1/pleiades-linux-amd64.tar.gz"
      sha256 "b75e3d6d0a134b539956e6731519d5da0f45ee53a99081c17edbf2f63f264e8d"
    end
  end

  def install
    bin.install "pleiades"
  end

  test do
    assert_match "Pleiades", shell_output("#{bin}/pleiades --help")
  end
end
