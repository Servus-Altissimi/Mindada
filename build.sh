#!/bin/sh

echo "Building Mindada..."
cargo +nightly build --release -Z build-std=core,compiler_builtins -Z build-std-features=compiler-builtins-mem --target x86_64-Mindada.json

if [ ! -f target/x86_64-Mindada/release/Mindada ]; then
    echo "Kernel binary not found."
    exit 1
fi

echo "Build successful!"
echo "Creating bootable ISO..."

# Create ISO directory struct & copies files
mkdir -p isofiles/boot/grub

# Copy kernel
cp target/x86_64-Mindada/release/Mindada isofiles/boot/kernel.bin

# Grub
cat > isofiles/boot/grub/grub.cfg << EOF
set timeout=0
set default=0

menuentry "Mindada" {
    multiboot /boot/kernel.bin
    boot
}
EOF

# ISO
grub-mkrescue -o Mindada.iso isofiles

echo ""
echo "Running in QEMU..."
qemu-system-x86_64 -cdrom Mindada.iso -serial stdio