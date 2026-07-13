class Pleiades < Formula
  desc "Provider-agnostic terminal AI assistant"
  homepage "https://github.com/CodWasTaken/Pleiades"
  head "https://github.com/CodWasTaken/Pleiades.git", branch: "master"
  license "MIT"
  version "1.0.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/CodWasTaken/Pleiades/releases/download/v1.0.0/pleiades-macos-arm64.tar.gz"
      sha256 "9bcf84bb6066505bd52e1bb70f02f0fb9f09aa4819da731a07fcbce20094ab59"
    else
      url "https://github.com/CodWasTaken/Pleiades/releases/download/v1.0.0/pleiades-macos-amd64.tar.gz"
      sha256 "fe73d3fd401ada3fdc9165388331ceac510fbbfb04c018bcf40a4927b6e6d101"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/CodWasTaken/Pleiades/releases/download/v1.0.0/pleiades-linux-arm64.tar.gz"
      sha256 "6021399a62e5aef717ad1b51196666b2e185f717227abffd41beebc7676e19ea"
    else
      url "https://github.com/CodWasTaken/Pleiades/releases/download/v1.0.0/pleiades-linux-amd64.tar.gz"
      sha256 "7a7ede273e5c37f0ab48dcaad086048b5a67b5818449816d673a01d9c5d619c4"
    end
  end

  def install
    bin.install "pleiades"
  end

  test do
    assert_match "Pleiades", shell_output("#{bin}/pleiades --help")
  end
end
