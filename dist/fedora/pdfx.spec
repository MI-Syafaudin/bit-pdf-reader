Name:           pdfx
Version:        0.1.0
Release:        1%{?dist}
Summary:        High-performance Terminal & CLI PDF Swiss Army Knife for Linux

License:        Apache-2.0
URL:            https://github.com/pdfx/pdfx
Source0:        %{url}/archive/v%{version}/%{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust >= 1.70.0
Recommends:     tesseract
Suggests:       tesseract-langpack-eng
Suggests:       tesseract-langpack-ind

%description
PDFX is a high-performance terminal & CLI PDF management and processing
Swiss Army Knife for Linux written in modern Rust. It provides a dual-mode
interface: an interactive dual-pane TUI powered by Ratatui and Crossterm,
and a comprehensive scriptable CLI with strict POSIX exit codes and atomic
file safety.

%prep
%autosetup -n %{name}-%{version}

%build
cargo build --release --locked

%check
cargo test --release --locked --workspace

%install
install -Dm755 target/release/pdfx %{buildroot}%{_bindir}/pdfx
install -Dm644 man/pdfx.1 %{buildroot}%{_mandir}/man1/pdfx.1
install -Dm644 dist/pdfx.desktop %{buildroot}%{_datadir}/applications/pdfx.desktop
install -Dm644 completions/pdfx.bash %{buildroot}%{_datadir}/bash-completion/completions/pdfx
install -Dm644 completions/_pdfx %{buildroot}%{_datadir}/zsh/site-functions/_pdfx
install -Dm644 completions/pdfx.fish %{buildroot}%{_datadir}/fish/vendor_completions.d/pdfx.fish

%files
%license LICENSE
%{_bindir}/pdfx
%{_mandir}/man1/pdfx.1*
%{_datadir}/applications/pdfx.desktop
%{_datadir}/bash-completion/completions/pdfx
%{_datadir}/zsh/site-functions/_pdfx
%{_datadir}/fish/vendor_completions.d/pdfx.fish

%changelog
* Thu Oct 01 2026 PDFX Contributors <info@pdfx.org> - 0.1.0-1
- Initial release of PDFX v0.1.0
