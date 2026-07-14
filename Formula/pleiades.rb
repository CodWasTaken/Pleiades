class Pleiades < Formula
  desc "Provider-agnostic autonomous terminal coding agent"
  homepage "https://github.com/CodWasTaken/Pleiades"
  head "https://github.com/CodWasTaken/Pleiades.git", branch: "master"
  license "MIT"
  version "2.0.0"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/CodWasTaken/Pleiades/releases/download/v2.0.0/pleiades-macos-arm64.tar.gz"
      sha256 "f1e730cae00d99e7eaada6fd83840df23e13590402e7d8b868c01b779b055264"
    else
      url "https://github.com/CodWasTaken/Pleiades/releases/download/v2.0.0/pleiades-macos-amd64.tar.gz"
      sha256 "5c841a7f9ffa8dea7e36320d703648aa0ad1e63ce4433f569eefca4e679f6bd3"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/CodWasTaken/Pleiades/releases/download/v2.0.0/pleiades-linux-arm64.tar.gz"
      sha256 "475d70f257751e851e9fae20f941ca9999915beba44b8123af6c3182b6dca59d"
    else
      url "https://github.com/CodWasTaken/Pleiades/releases/download/v2.0.0/pleiades-linux-amd64.tar.gz"
      sha256 "a016e3f78b2ee314a067e7fda1ee2c1b66642a69e06940f5d82c59fb60c6ea0e"
    end
  end

  def install
    bin.install "pleiades"
  end

  test do
    assert_match "Pleiades", shell_output("#{bin}/pleiades --help")
  end
end
