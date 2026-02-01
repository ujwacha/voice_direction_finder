(use-modules (gnu)
	     (guix profiles)
	     (guix packages))

(use-package-modules base
 		     commencement
		     fontutils
		     freedesktop
		     gl
		     linux
		     pkg-config
		     rust
		     tls
		     vulkan
		     xdisorg
		     xml)


(define vdf-packages  (list
		       ;; Rust stuff
 		       rust
		       (list rust "cargo")
		       (list rust "rust-src")
		       (list rust "tools")
		       rust-analyzer

		       ;; GNU coretuils
		       gcc-toolchain
		       (list gcc-toolchain "debug")
		       (list gcc-toolchain "static")
		       coreutils
		       glibc
		       openssl
		       pkg-config

		       ;; Graphics and Audio libs
		       alsa-lib
		       expat
		       fontconfig
		       freetype
		       libxkbcommon
		       mesa
		       vulkan-loader
		       wayland
		       wayland-protocols
		       ))

(packages->manifest vdf-packages)
