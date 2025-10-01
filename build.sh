#!/bin/sh

echo "Building Mindada!"
echo "Assembling boot.s"
as --64 boot.s -o boot.o

if [ ! -f boot.o ]; then
    echo "Failed to assemble boot.s"
    exit 1
fi

echo "Building kernel"
cargo +nightly build --release \
    -Z build-std=core \
    --target x86_64-Mindada.json

if [ ! -f target/x86_64-Mindada/release/libMindada.a ]; then
    echo "Kernel binary not found."
    exit 1
fi

echo "Linking kernel"
ld -n -o kernel.bin -T linker.ld boot.o target/x86_64-Mindada/release/libMindada.a --print-map > link.map 2>&1 || true
ld -n -o kernel.bin -T linker.ld boot.o target/x86_64-Mindada/release/libMindada.a

if [ ! -f kernel.bin ]; then
    echo "Linking failed."
    exit 1
fi

echo "Verifying multiboot header"
HEADER=$(od -An -t x4 -N 12 kernel.bin | tr -d ' \n')
if echo "$HEADER" | grep -q "1badb002"; then
    echo "Multiboot header found"
    echo "Header bytes: $HEADER"
else
    echo "Multiboot header not found at start of binary"
    echo "First 12 bytes: $HEADER"
    echo "Expected to see: 1badb002 00000000 e4524ffb (or similar)"
fi

echo "Build successfully"
echo "Kernel size: $(stat -f%z kernel.bin 2>/dev/null || stat -c%s kernel.bin) bytes"

echo "Creating bootable .iso"

rm -rf isofiles
rm -f Mindada.iso

mkdir -p isofiles/boot/grub

cp kernel.bin isofiles/boot/kernel.bin

cat > isofiles/boot/grub/grub.cfg << 'EOF'
set timeout=0
set default=0

menuentry "Mindada" {
    multiboot /boot/kernel.bin
    boot
}
EOF

grub-mkrescue -o Mindada.iso isofiles 2>&1

if [ ! -f Mindada.iso ]; then
    echo "Failed to create .iso"
    exit 1
fi

echo ""
echo "ISO created successfully"
echo "Running in QEMU"
echo ""

qemu-system-x86_64 \
    -cdrom Mindada.iso \
    -serial stdio \
    -netdev user,id=net0 \
    -device e1000,netdev=net0 \
    -m 128M \
    -d cpu_reset,guest_errors