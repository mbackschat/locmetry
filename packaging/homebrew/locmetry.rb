class Locmetry < Formula
  desc "Measure code with embedded Tokei and reviewed classification rules"
  homepage "https://github.com/mbackschat/locmetry"
  url "https://github.com/mbackschat/locmetry/archive/refs/tags/v{{version}}.tar.gz"
  sha256 "{{sha256}}"
  license any_of: ["MIT", "Apache-2.0"]
  head "https://github.com/mbackschat/locmetry.git", branch: "main"

  # Tokei is compiled in, so the installed executable has no runtime formula dependencies.
  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    (testpath/"project/src/lib.rs").write "pub fn answer() -> u32 {\n    42\n}\n"
    (testpath/"project/tests/answer.rs").write "#[test]\nfn answer() {\n    assert_eq!(2 + 2, 4);\n}\n"
    (testpath/"project/.locmetryrc").write <<~TOML
      version = 1

      [[rules]]
      id = "library"
      match = ["src/**"]
      purpose = "production"
    TOML

    # An empty PATH guards against a runtime dependency on any other executable.
    output = with_env(PATH: "") { shell_output("#{bin}/locmetry --root #{testpath}/project") }
    purposes = JSON.parse(output)["summary"]["source_purpose"]
    assert_equal 3, purposes["production"]["code"]
    assert_equal 4, purposes["test"]["code"]
    refute purposes.key?("unclassified")
  end
end
