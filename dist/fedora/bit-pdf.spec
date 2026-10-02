Name:           bit-pdf
Version:        0.1.0
Release:        1%{?dist}
Summary:        Bit PDF Reader — High-performance Terminal & CLI PDF Swiss Army Knife for Linux

License:        Apache-2.0
URL:            https://github.com/MI-Syafaudin/bit-pdf-reader
Source0:        %{url}/archive/v%{version}/%{name}-%{version}.tar.gz

BuildRequires:  cargo
BuildRequires:  rust >= 1.70.0
Provides:       pdfx
Recommends:     tesseract
Suggests:       tesseract-langpack-eng
Suggests:       tesseract-langpack-ind

%description
Bit PDF Reader (created by MI_Syafaudin) is a high-performance terminal & CLI PDF
management and processing Swiss Army Knife for Linux written in modern Rust.
It provides a dual-mode interface: an interactive dual-pane TUI powered by Ratatui
and Crossterm, and a comprehensive scriptable CLI with strict POSIX exit codes and
atomic file safety.

%prep
%autosetup -n %{name}-%{version}

%build
cargo build --release --locked

%check
cargo test --release --locked --workspace

%install
install -Dm755 target/release/bit-pdf %{buildroot}%{_bindir}/bit-pdf
install -Dm755 target/release/pdfx %{buildroot}%{_bindir}/pdfx
install -Dm644 man/bit-pdf.1 %{buildroot}%{_mandir}/man1/bit-pdf.1
install -Dm644 man/pdfx.1 %{buildroot}%{_mandir}/man1/pdfx.1
install -Dm644 dist/bit-pdf.desktop %{buildroot}%{_datadir}/applications/bit-pdf.desktop
install -Dm644 dist/pdfx.desktop %{buildroot}%{_datadir}/applications/pdfx.desktop
install -Dm644 completions/bit-pdf.bash %{buildroot}%{_datadir}/bash-completion/completions/bit-pdf
install -Dm644 completions/pdfx.bash %{buildroot}%{_datadir}/bash-completion/completions/pdfx
install -Dm644 completions/_bit-pdf %{buildroot}%{_datadir}/zsh/site-functions/_bit-pdf
install -Dm644 completions/_pdfx %{buildroot}%{_datadir}/zsh/site-functions/_pdfx
install -Dm644 completions/bit-pdf.fish %{buildroot}%{_datadir}/fish/vendor_completions.d/bit-pdf.fish
install -Dm644 completions/pdfx.fish %{buildroot}%{_datadir}/fish/vendor_completions.d/pdfx.fish

%files
%license LICENSE
%{_bindir}/bit-pdf
%{_bindir}/pdfx
%{_mandir}/man1/bit-pdf.1*
%{_mandir}/man1/pdfx.1*
%{_datadir}/applications/bit-pdf.desktop
%{_datadir}/applications/pdfx.desktop
%{_datadir}/bash-completion/completions/bit-pdf
%{_datadir}/bash-completion/completions/pdfx
%{_datadir}/zsh/site-functions/_bit-pdf
%{_datadir}/zsh/site-functions/_pdfx
%{_datadir}/fish/vendor_completions.d/bit-pdf.fish
%{_datadir}/fish/vendor_completions.d/pdfx.fish

%changelog
* Thu Oct 01 2026 MI_Syafaudin <https://github.com/MI-Syafaudin> - 0.1.0-1
- Initial release of Bit PDF Reader v0.1.0
