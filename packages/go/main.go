// Package main provides a Go wrapper for the mille architecture checker.
//
// Install:
//
//	go install github.com/makinzm/mille/packages/go@latest
//
// This wrapper downloads the pre-built mille binary from GitHub Releases on
// first use, caches it under ~/.mille/bin/<version>/, and delegates all
// arguments to it.
package main

import (
	"archive/tar"
	"archive/zip"
	"compress/gzip"
	"errors"
	"fmt"
	"io"
	"net/http"
	"os"
	"os/exec"
	"path/filepath"
	"runtime"
)

// version must match the mille release tag (without the leading "v").
const (
	version = "0.0.1"
	repo    = "makinzm/mille"
)

// targetTriple maps the current OS/arch to the Rust target triple used in
// release artifact names.
func targetTriple() (string, error) {
	switch runtime.GOOS + "/" + runtime.GOARCH {
	case "linux/amd64":
		return "x86_64-unknown-linux-gnu", nil
	case "linux/arm64":
		return "aarch64-unknown-linux-gnu", nil
	case "darwin/amd64":
		return "x86_64-apple-darwin", nil
	case "darwin/arm64":
		return "aarch64-apple-darwin", nil
	case "windows/amd64":
		return "x86_64-pc-windows-msvc", nil
	default:
		return "", fmt.Errorf("unsupported platform: %s/%s", runtime.GOOS, runtime.GOARCH)
	}
}

// cacheDir returns (and creates if needed) the versioned cache directory.
func cacheDir() (string, error) {
	home, err := os.UserHomeDir()
	if err != nil {
		return "", err
	}
	dir := filepath.Join(home, ".mille", "bin", version)
	return dir, os.MkdirAll(dir, 0o755)
}

// binaryName returns the OS-specific name of the mille executable.
func binaryName() string {
	if runtime.GOOS == "windows" {
		return "mille.exe"
	}
	return "mille"
}

// downloadAndExtract fetches the release archive and extracts the mille binary
// to destDir. Returns the path to the extracted binary.
func downloadAndExtract(destDir string) (string, error) {
	target, err := targetTriple()
	if err != nil {
		return "", err
	}

	isWindows := runtime.GOOS == "windows"
	var ext string
	if isWindows {
		ext = "zip"
	} else {
		ext = "tar.gz"
	}

	archiveName := fmt.Sprintf("mille-%s-%s.%s", version, target, ext)
	url := fmt.Sprintf("https://github.com/%s/releases/download/v%s/%s", repo, version, archiveName)

	resp, err := http.Get(url) //nolint:noctx
	if err != nil {
		return "", fmt.Errorf("download %s: %w", url, err)
	}
	defer resp.Body.Close()
	if resp.StatusCode != http.StatusOK {
		return "", fmt.Errorf("download %s: HTTP %d", url, resp.StatusCode)
	}

	binPath := filepath.Join(destDir, binaryName())

	if isWindows {
		if err := extractZip(resp.Body, binPath); err != nil {
			return "", err
		}
	} else {
		if err := extractTarGz(resp.Body, binPath); err != nil {
			return "", err
		}
	}
	return binPath, nil
}

// extractTarGz reads a .tar.gz stream and writes the "mille" entry to dest.
func extractTarGz(r io.Reader, dest string) error {
	gr, err := gzip.NewReader(r)
	if err != nil {
		return fmt.Errorf("gzip: %w", err)
	}
	defer gr.Close()

	tr := tar.NewReader(gr)
	for {
		hdr, err := tr.Next()
		if errors.Is(err, io.EOF) {
			break
		}
		if err != nil {
			return fmt.Errorf("tar: %w", err)
		}
		if hdr.Name == "mille" {
			return writeExecutable(tr, dest)
		}
	}
	return errors.New("mille not found in archive")
}

// extractZip saves the response body to a temp file and extracts "mille.exe".
func extractZip(r io.Reader, dest string) error {
	tmp, err := os.CreateTemp("", "mille-*.zip")
	if err != nil {
		return err
	}
	defer os.Remove(tmp.Name())

	if _, err := io.Copy(tmp, r); err != nil {
		tmp.Close()
		return err
	}
	name := tmp.Name()
	tmp.Close()

	zr, err := zip.OpenReader(name)
	if err != nil {
		return fmt.Errorf("zip: %w", err)
	}
	defer zr.Close()

	for _, f := range zr.File {
		if f.Name == "mille.exe" {
			rc, err := f.Open()
			if err != nil {
				return err
			}
			defer rc.Close()
			return writeExecutable(rc, dest)
		}
	}
	return errors.New("mille.exe not found in archive")
}

// writeExecutable writes src to path with executable permissions.
func writeExecutable(src io.Reader, path string) error {
	f, err := os.OpenFile(path, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, 0o755)
	if err != nil {
		return err
	}
	_, err = io.Copy(f, src)
	if cerr := f.Close(); cerr != nil && err == nil {
		err = cerr
	}
	return err
}

func run() error {
	dir, err := cacheDir()
	if err != nil {
		return fmt.Errorf("cache dir: %w", err)
	}

	binPath := filepath.Join(dir, binaryName())
	if _, err := os.Stat(binPath); errors.Is(err, os.ErrNotExist) {
		fmt.Fprintf(os.Stderr, "Downloading mille v%s...\n", version)
		binPath, err = downloadAndExtract(dir)
		if err != nil {
			return fmt.Errorf("install: %w", err)
		}
	}

	cmd := exec.Command(binPath, os.Args[1:]...)
	cmd.Stdin = os.Stdin
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd.Run()
}

func main() {
	if err := run(); err != nil {
		var exitErr *exec.ExitError
		if errors.As(err, &exitErr) {
			os.Exit(exitErr.ExitCode())
		}
		fmt.Fprintln(os.Stderr, "Error:", err)
		os.Exit(1)
	}
}
