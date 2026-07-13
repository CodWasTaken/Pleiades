class Pleiades < Formula
  desc "Provider-agnostic terminal AI assistant"
  homepage "https://github.com/CodWasTaken/Pleiades"
  head "https://github.com/CodWasTaken/Pleiades.git", branch: "master"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "crates/pleiades-cli")
  end

  test do
    assert_match "Pleiades", shell_output("#{bin}/pleiades --help")
  end
end
