class Pleiades < Formula
  desc "Provider-agnostic terminal AI assistant"
  homepage "https://github.com/CodWasTaken/Pleiades"
  head "https://github.com/CodWasTaken/Pleiades.git", branch: "master"
  license "MIT"
  version "1.0.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/CodWasTaken/Pleiades/releases/download/v1.0.0/pleiades-macos-arm64.tar.gz"
      sha256 "6d4c919ccb64c793d268bcd3dca82549fc5956af08a12d076e039f5cc663ebdd"
    else
      url "https://github.com/CodWasTaken/Pleiades/releases/download/v1.0.0/pleiades-macos-amd64.tar.gz"
      sha256 "dd89dc30e4854a66086c550e107d10039a26c23cc2e1025a1d55a27b7ecae0fc"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/CodWasTaken/Pleiades/releases/download/v1.0.0/pleiades-linux-arm64.tar.gz"
      sha256 "0293bc408525aec75671ec0b04c745625047e85bf5c918e663060eb58cf63e13"
    else
      url "https://github.com/CodWasTaken/Pleiades/releases/download/v1.0.0/pleiades-linux-amd64.tar.gz"
      sha256 "dd4bb576caff8c26058500434b07725bd3e41c57510e09632d63b13b300b2fd1"
    end
  end

  def install
    bin.install "pleiades"
  end

  test do
    assert_match "Pleiades", shell_output("#{bin}/pleiades --help")
  end
end
