#!/bin/bash

set -e

# Default SSH port
port=22

# Parse command line options
while getopts "p:h" opt; do
    case $opt in
        p)
            port="$OPTARG"
            ;;
        h)
            echo "Usage: $0 [-p <port>] <user>@<host>"
            echo "  -p <port>   SSH port (default: 22)"
            echo "  -h          Show this help message"
            exit 0
            ;;
        \?)
            echo "Invalid option: -$OPTARG" >&2
            echo "Usage: $0 [-p <port>] <user>@<host>" >&2
            exit 1
            ;;
        :)
            echo "Option -$OPTARG requires an argument" >&2
            exit 1
            ;;
    esac
done

# Shift past the options to get the positional argument
shift $((OPTIND - 1))

# Check that user@host is provided
if [ $# -lt 1 ]; then
    echo "Error: Missing required argument <user>@<host>" >&2
    echo "Usage: $0 [-p <port>] <user>@<host>" >&2
    exit 1
fi

destination="$1"

# Validate destination format (must contain @)
if [[ ! "$destination" =~ ^[^@]+@[^@]+$ ]]; then
    echo "Error: Destination must be in the format <user>@<host>" >&2
    exit 1
fi

# Build the remote server
echo "Building remote server for x86_64-unknown-linux-musl..."
cargo build --release --target x86_64-unknown-linux-musl -p remote_server

# Compress the binary
echo "Compressing binary..."
gzip -c target/x86_64-unknown-linux-musl/release/remote_server > remote_server.gz

# Copy to remote host
echo "Copying to $destination (port $port)..."
scp -P "$port" remote_server.gz "$destination:~/.zed_server/zed-remote-server-dev-build.gz"

# Extract and set permissions on remote host
echo "Setting up on remote host..."
ssh -p "$port" "$destination" "gunzip -f ~/.zed_server/zed-remote-server-dev-build.gz && chmod +x ~/.zed_server/zed-remote-server-dev-build"

# Cleanup local compressed file
rm -f remote_server.gz

echo "Done! Remote server deployed to $destination"
