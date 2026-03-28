class Tos < Formula
  desc "Tactical Operating System (Beta-0)"
  homepage "https://tos-project.org"
  url "https://github.com/tos-project/tos/archive/refs/tags/v0.1.0-beta.0.tar.gz"
  sha256 "0000000000000000000000000000000000000000000000000000000000000000"
  license "GPL-3.0-or-later"

  depends_on "rust" => :build
  depends_on "node" => :build

  def install
    # Build Svelte Web UI
    cd "svelte_ui" do
      system "npm", "install"
      system "npm", "run", "build"
    end

    # Build Cargo Daemons and Brain
    system "make", "build-services"

    # Install main binaries
    bin.install "target/release/tos-brain"
    bin.install "target/release/tos"
    bin.install "target/release/logs"
  end

  test do
    # Verify the CLI responds
    system "#{bin}/tos", "--version"
  end
end
