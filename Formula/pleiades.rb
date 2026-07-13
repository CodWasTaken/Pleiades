class Pleiades < Formula
  desc "Provider-agnostic terminal AI assistant"
  homepage "https://github.com/CodWasTaken/Pleiades"
  head "https://github.com/CodWasTaken/Pleiades.git", branch: "master"
  license "MIT"
  version "1.1.1"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/CodWasTaken/Pleiades/releases/download/v1.1.1/pleiades-macos-arm64.tar.gz"
      sha256 "b6ce3c68bb67beba5704c274c17fbd142d8dbd0533103b7d6686220a8c5f590f"
    else
      url "https://github.com/CodWasTaken/Pleiades/releases/download/v1.1.1/pleiades-macos-amd64.tar.gz"
      sha256 "91690bd6ba20117051a015a6e3ec04dc2d00ce65368681e433af4b7030c67f35"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/CodWasTaken/Pleiades/releases/download/v1.1.1/pleiades-linux-arm64.tar.gz"
      sha256 "1ee4371803eef2b9cd74cd0ffd7af45c09921b91b10ee36677426120d6957055"
    else
      url "https://github.com/CodWasTaken/Pleiades/releases/download/v1.1.1/pleiades-linux-amd64.tar.gz"
      sha256 "a316ffd47d3986ea5aca577b1b7b9f5e10c711de1af614ba487558abff6263c5"
    end
  end

  def install
    bin.install "pleiades"
  end

  test do
    assert_match "Pleiades", shell_output("#{bin}/pleiades --help")
  end
end
