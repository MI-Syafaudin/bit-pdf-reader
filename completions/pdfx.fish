# Print an optspec for argparse to handle cmd's options that are independent of any subcommand.
function __fish_pdfx_global_optspecs
    string join \n d/debug json h/help V/version
end

function __fish_pdfx_needs_command
    # Figure out if the current invocation already has a command.
    set -l cmd (commandline -opc)
    set -e cmd[1]
    argparse -s (__fish_pdfx_global_optspecs) -- $cmd 2>/dev/null
    or return
    if set -q argv[1]
        # Also print the command, so this can be used to figure out what it is.
        echo $argv[1]
        return 1
    end
    return 0
end

function __fish_pdfx_using_subcommand
    set -l cmd (__fish_pdfx_needs_command)
    test -z "$cmd"
    and return 1
    contains -- $cmd[1] $argv
end

complete -c pdfx -n "__fish_pdfx_needs_command" -s d -l debug -d 'Enable verbose debug logging'
complete -c pdfx -n "__fish_pdfx_needs_command" -l json -d 'Output result in machine-readable JSON format'
complete -c pdfx -n "__fish_pdfx_needs_command" -s h -l help -d 'Print help (see more with \'--help\')'
complete -c pdfx -n "__fish_pdfx_needs_command" -s V -l version -d 'Print version'
complete -c pdfx -n "__fish_pdfx_needs_command" -f -a "open" -d 'Open interactive PDF TUI viewer'
complete -c pdfx -n "__fish_pdfx_needs_command" -f -a "info" -d 'Inspect document metadata, version, encryption, and page dimensions'
complete -c pdfx -n "__fish_pdfx_needs_command" -f -a "merge" -d 'Merge multiple PDF files into a single document'
complete -c pdfx -n "__fish_pdfx_needs_command" -f -a "split" -d 'Split a PDF document into page ranges or individual parts'
complete -c pdfx -n "__fish_pdfx_needs_command" -f -a "extract" -d 'Extract specific pages from a PDF into a new document'
complete -c pdfx -n "__fish_pdfx_needs_command" -f -a "rotate" -d 'Rotate pages in a PDF document'
complete -c pdfx -n "__fish_pdfx_needs_command" -f -a "delete" -d 'Delete specific pages from a PDF document'
complete -c pdfx -n "__fish_pdfx_needs_command" -f -a "reorder" -d 'Reorder pages in a PDF document'
complete -c pdfx -n "__fish_pdfx_needs_command" -f -a "text" -d 'Extract text content from a PDF document'
complete -c pdfx -n "__fish_pdfx_needs_command" -f -a "compress" -d 'Optimize and compress PDF document streams'
complete -c pdfx -n "__fish_pdfx_needs_command" -f -a "ocr" -d 'Run OCR on scanned PDF pages to generate searchable document'
complete -c pdfx -n "__fish_pdfx_needs_command" -f -a "clean" -d 'Clean recent files and job history cache'
complete -c pdfx -n "__fish_pdfx_needs_command" -f -a "completions" -d 'Generate shell completion scripts (bash, zsh, fish, powershell)'
complete -c pdfx -n "__fish_pdfx_needs_command" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
complete -c pdfx -n "__fish_pdfx_using_subcommand open" -s d -l debug -d 'Enable verbose debug logging'
complete -c pdfx -n "__fish_pdfx_using_subcommand open" -l json -d 'Output result in machine-readable JSON format'
complete -c pdfx -n "__fish_pdfx_using_subcommand open" -s h -l help -d 'Print help'
complete -c pdfx -n "__fish_pdfx_using_subcommand info" -s d -l debug -d 'Enable verbose debug logging'
complete -c pdfx -n "__fish_pdfx_using_subcommand info" -l json -d 'Output result in machine-readable JSON format'
complete -c pdfx -n "__fish_pdfx_using_subcommand info" -s h -l help -d 'Print help'
complete -c pdfx -n "__fish_pdfx_using_subcommand merge" -s o -l output -d 'Target destination output file' -r -F
complete -c pdfx -n "__fish_pdfx_using_subcommand merge" -s d -l debug -d 'Enable verbose debug logging'
complete -c pdfx -n "__fish_pdfx_using_subcommand merge" -l json -d 'Output result in machine-readable JSON format'
complete -c pdfx -n "__fish_pdfx_using_subcommand merge" -s h -l help -d 'Print help'
complete -c pdfx -n "__fish_pdfx_using_subcommand split" -s p -l pages -d 'Page range (e.g. \'1-5\', \'1,3,5\', \'odd\', \'even\')' -r
complete -c pdfx -n "__fish_pdfx_using_subcommand split" -s o -l output-dir -d 'Destination directory for split files (defaults to current dir)' -r -F
complete -c pdfx -n "__fish_pdfx_using_subcommand split" -s d -l debug -d 'Enable verbose debug logging'
complete -c pdfx -n "__fish_pdfx_using_subcommand split" -l json -d 'Output result in machine-readable JSON format'
complete -c pdfx -n "__fish_pdfx_using_subcommand split" -s h -l help -d 'Print help'
complete -c pdfx -n "__fish_pdfx_using_subcommand extract" -s p -l pages -d 'Page range to extract (e.g. \'1-3\', \'5,8\')' -r
complete -c pdfx -n "__fish_pdfx_using_subcommand extract" -s o -l output -d 'Target output PDF file' -r -F
complete -c pdfx -n "__fish_pdfx_using_subcommand extract" -s d -l debug -d 'Enable verbose debug logging'
complete -c pdfx -n "__fish_pdfx_using_subcommand extract" -l json -d 'Output result in machine-readable JSON format'
complete -c pdfx -n "__fish_pdfx_using_subcommand extract" -s h -l help -d 'Print help'
complete -c pdfx -n "__fish_pdfx_using_subcommand rotate" -s p -l pages -d 'Page range to rotate (\'all\', \'1-5\', \'odd\')' -r
complete -c pdfx -n "__fish_pdfx_using_subcommand rotate" -s a -l degrees -d 'Rotation angle in degrees: 90, 180, or 270' -r
complete -c pdfx -n "__fish_pdfx_using_subcommand rotate" -s o -l output -d 'Target output PDF file' -r -F
complete -c pdfx -n "__fish_pdfx_using_subcommand rotate" -s d -l debug -d 'Enable verbose debug logging'
complete -c pdfx -n "__fish_pdfx_using_subcommand rotate" -l json -d 'Output result in machine-readable JSON format'
complete -c pdfx -n "__fish_pdfx_using_subcommand rotate" -s h -l help -d 'Print help'
complete -c pdfx -n "__fish_pdfx_using_subcommand delete" -s p -l pages -d 'Page range to delete (e.g. \'1,3\', \'5-10\')' -r
complete -c pdfx -n "__fish_pdfx_using_subcommand delete" -s o -l output -d 'Target output PDF file' -r -F
complete -c pdfx -n "__fish_pdfx_using_subcommand delete" -s y -l yes -d 'Confirm deletion without interactive prompt'
complete -c pdfx -n "__fish_pdfx_using_subcommand delete" -s d -l debug -d 'Enable verbose debug logging'
complete -c pdfx -n "__fish_pdfx_using_subcommand delete" -l json -d 'Output result in machine-readable JSON format'
complete -c pdfx -n "__fish_pdfx_using_subcommand delete" -s h -l help -d 'Print help'
complete -c pdfx -n "__fish_pdfx_using_subcommand reorder" -s s -l order -d 'New page order as comma-separated 1-indexed numbers (e.g. \'3,1,2,4\')' -r
complete -c pdfx -n "__fish_pdfx_using_subcommand reorder" -s o -l output -d 'Target output PDF file' -r -F
complete -c pdfx -n "__fish_pdfx_using_subcommand reorder" -s d -l debug -d 'Enable verbose debug logging'
complete -c pdfx -n "__fish_pdfx_using_subcommand reorder" -l json -d 'Output result in machine-readable JSON format'
complete -c pdfx -n "__fish_pdfx_using_subcommand reorder" -s h -l help -d 'Print help'
complete -c pdfx -n "__fish_pdfx_using_subcommand text" -s p -l pages -d 'Page range to extract (\'all\', \'1-5\', \'odd\')' -r
complete -c pdfx -n "__fish_pdfx_using_subcommand text" -s f -l format -d 'Output text format: \'plain\', \'json\', or \'jsonl\'' -r
complete -c pdfx -n "__fish_pdfx_using_subcommand text" -s o -l output -d 'Destination file for text (prints to stdout if omitted)' -r -F
complete -c pdfx -n "__fish_pdfx_using_subcommand text" -s d -l debug -d 'Enable verbose debug logging'
complete -c pdfx -n "__fish_pdfx_using_subcommand text" -l json -d 'Output result in machine-readable JSON format'
complete -c pdfx -n "__fish_pdfx_using_subcommand text" -s h -l help -d 'Print help'
complete -c pdfx -n "__fish_pdfx_using_subcommand compress" -s o -l output -d 'Target compressed output file' -r -F
complete -c pdfx -n "__fish_pdfx_using_subcommand compress" -s d -l debug -d 'Enable verbose debug logging'
complete -c pdfx -n "__fish_pdfx_using_subcommand compress" -l json -d 'Output result in machine-readable JSON format'
complete -c pdfx -n "__fish_pdfx_using_subcommand compress" -s h -l help -d 'Print help'
complete -c pdfx -n "__fish_pdfx_using_subcommand ocr" -s l -l lang -d 'OCR language (e.g. \'eng\', \'ind\', \'eng+ind\')' -r
complete -c pdfx -n "__fish_pdfx_using_subcommand ocr" -s p -l pages -d 'Page range to perform OCR on' -r
complete -c pdfx -n "__fish_pdfx_using_subcommand ocr" -s o -l output -d 'Target output searchable PDF' -r -F
complete -c pdfx -n "__fish_pdfx_using_subcommand ocr" -s d -l debug -d 'Enable verbose debug logging'
complete -c pdfx -n "__fish_pdfx_using_subcommand ocr" -l json -d 'Output result in machine-readable JSON format'
complete -c pdfx -n "__fish_pdfx_using_subcommand ocr" -s h -l help -d 'Print help'
complete -c pdfx -n "__fish_pdfx_using_subcommand clean" -l history -d 'Clear recent opened files history'
complete -c pdfx -n "__fish_pdfx_using_subcommand clean" -l cache -d 'Clear cached metadata and temporary files'
complete -c pdfx -n "__fish_pdfx_using_subcommand clean" -l all -d 'Clear all recent files, history, and cache'
complete -c pdfx -n "__fish_pdfx_using_subcommand clean" -s d -l debug -d 'Enable verbose debug logging'
complete -c pdfx -n "__fish_pdfx_using_subcommand clean" -l json -d 'Output result in machine-readable JSON format'
complete -c pdfx -n "__fish_pdfx_using_subcommand clean" -s h -l help -d 'Print help'
complete -c pdfx -n "__fish_pdfx_using_subcommand completions" -s d -l debug -d 'Enable verbose debug logging'
complete -c pdfx -n "__fish_pdfx_using_subcommand completions" -l json -d 'Output result in machine-readable JSON format'
complete -c pdfx -n "__fish_pdfx_using_subcommand completions" -s h -l help -d 'Print help'
complete -c pdfx -n "__fish_pdfx_using_subcommand help; and not __fish_seen_subcommand_from open info merge split extract rotate delete reorder text compress ocr clean completions help" -f -a "open" -d 'Open interactive PDF TUI viewer'
complete -c pdfx -n "__fish_pdfx_using_subcommand help; and not __fish_seen_subcommand_from open info merge split extract rotate delete reorder text compress ocr clean completions help" -f -a "info" -d 'Inspect document metadata, version, encryption, and page dimensions'
complete -c pdfx -n "__fish_pdfx_using_subcommand help; and not __fish_seen_subcommand_from open info merge split extract rotate delete reorder text compress ocr clean completions help" -f -a "merge" -d 'Merge multiple PDF files into a single document'
complete -c pdfx -n "__fish_pdfx_using_subcommand help; and not __fish_seen_subcommand_from open info merge split extract rotate delete reorder text compress ocr clean completions help" -f -a "split" -d 'Split a PDF document into page ranges or individual parts'
complete -c pdfx -n "__fish_pdfx_using_subcommand help; and not __fish_seen_subcommand_from open info merge split extract rotate delete reorder text compress ocr clean completions help" -f -a "extract" -d 'Extract specific pages from a PDF into a new document'
complete -c pdfx -n "__fish_pdfx_using_subcommand help; and not __fish_seen_subcommand_from open info merge split extract rotate delete reorder text compress ocr clean completions help" -f -a "rotate" -d 'Rotate pages in a PDF document'
complete -c pdfx -n "__fish_pdfx_using_subcommand help; and not __fish_seen_subcommand_from open info merge split extract rotate delete reorder text compress ocr clean completions help" -f -a "delete" -d 'Delete specific pages from a PDF document'
complete -c pdfx -n "__fish_pdfx_using_subcommand help; and not __fish_seen_subcommand_from open info merge split extract rotate delete reorder text compress ocr clean completions help" -f -a "reorder" -d 'Reorder pages in a PDF document'
complete -c pdfx -n "__fish_pdfx_using_subcommand help; and not __fish_seen_subcommand_from open info merge split extract rotate delete reorder text compress ocr clean completions help" -f -a "text" -d 'Extract text content from a PDF document'
complete -c pdfx -n "__fish_pdfx_using_subcommand help; and not __fish_seen_subcommand_from open info merge split extract rotate delete reorder text compress ocr clean completions help" -f -a "compress" -d 'Optimize and compress PDF document streams'
complete -c pdfx -n "__fish_pdfx_using_subcommand help; and not __fish_seen_subcommand_from open info merge split extract rotate delete reorder text compress ocr clean completions help" -f -a "ocr" -d 'Run OCR on scanned PDF pages to generate searchable document'
complete -c pdfx -n "__fish_pdfx_using_subcommand help; and not __fish_seen_subcommand_from open info merge split extract rotate delete reorder text compress ocr clean completions help" -f -a "clean" -d 'Clean recent files and job history cache'
complete -c pdfx -n "__fish_pdfx_using_subcommand help; and not __fish_seen_subcommand_from open info merge split extract rotate delete reorder text compress ocr clean completions help" -f -a "completions" -d 'Generate shell completion scripts (bash, zsh, fish, powershell)'
complete -c pdfx -n "__fish_pdfx_using_subcommand help; and not __fish_seen_subcommand_from open info merge split extract rotate delete reorder text compress ocr clean completions help" -f -a "help" -d 'Print this message or the help of the given subcommand(s)'
