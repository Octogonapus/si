#!/bin/bash

# vars
GITROOT="$(git rev-parse --show-toplevel)"
PACKAGEDIR="$GITROOT/cyclone-pkg"
ROOTFS="$PACKAGEDIR/cyclone-rootfs.ext4"
ROOTFSMOUNT="$PACKAGEDIR/rootfs"
GUESTDISK="/rootfs"
INITSCRIPT="$PACKAGEDIR/init.sh"

# create disk and mount to a known locations
sudo rm -rf $PACKAGEDIR
mkdir -p $ROOTFSMOUNT $KERNELMOUNT
dd if=/dev/zero of=$ROOTFS bs=1M count=1024
mkfs.ext4 $ROOTFS
sudo mount $ROOTFS $ROOTFSMOUNT

# create our script to add an init system to our container image
cat << EOL > $INITSCRIPT
apk add openrc mingetty openssh

# Make sure special file systems are mounted on boot:
rc-update add devfs boot
rc-update add procfs boot
rc-update add sysfs boot
rc-update add local boot

mkdir -p /root/.ssh/
echo "ssh-rsa AAAAB3NzaC1yc2EAAAADAQABAAABgQDLjOIStmAZ0kLEUQNem4nssCr9TYxcTQ7+N7SoI9A2Rvr8S1gAu4f48OxuAtsLemW3UFUQZSFGyJwMw6VXF7MTASv4/d61zD/5kWmkacNlcwKxyOV31wV1zwG/genCTgIOnG34v81VhHWr8t8aJN9+7rT0hKy9Xhkb3NvLlcv2J4U4+zgoic+W0lSDmApBnMCq5XadEUYyJ/NxEcDxcDp1nbaSNrl/Iy1l0p3VAsHF6Rfpjbh8z1JWcF/CKDAQuou22XA7cgnkN1RJsf1d5Y/czp0twBiRZtXJHC1csNlM/O5jdI1Nh90rzpYEgtURlP9+ABdNWV70tc7A8QZ0d6Sn3Yail6FRlLWFsY4mHrUPbyUQkg3Y4LWSrjz7hC5jjjNkl0mUP5gHlleGoJr8cli5M1Hl2MFD4vlbmHosj6+5Fs18YCVDRrxyeyLjLnDp8SHP9BDDGhBQMprfB3C1v1yiiOm6ZZTyztOx+7tLsaWc9MgCfqIoT9L2/l6m9Yt2zcE= john@Threadripper" >> /root/.ssh/authorized_keys
apk add openssh
rc-update add sshd

# Then, copy the newly configured system to the rootfs image:
for d in bin etc lib root sbin usr nix; do tar c "/\${d}" | tar x -C ${GUESTDISK}; done
for dir in dev proc run sys var; do mkdir ${GUESTDISK}/\${dir}; done

# autologin
echo "ttyS0::respawn:/sbin/mingetty --autologin root --noclear ttyS0" >> ${GUESTDISK}/etc/inittab
sed -i 's/root:*::0:::::/root:::0:::::/g' $GUESTDISK/etc/shadow

# autostart cyclone
cat << EOF > /rootfs/etc/init.d/cyclone
#!/sbin/openrc-run

name="cyclone"
description="Cyclone"
supervisor="supervise-daemon"
command="cyclone"
command_args="--bind-vsock 3:52 --decryption-key /dev.decryption.key --lang-server /usr/local/bin/lang-js --enable-watch --limit-requests 1 --watch-timeout 10 --enable-ping --enable-resolver --enable-action-run -vvvv >> /cyclone.log"
pidfile="/run/agent.pid"
EOF

chmod +x ${GUESTDISK}/usr/local/bin/cyclone
chmod +x ${GUESTDISK}/usr/local/bin/lang-js
chmod +x ${GUESTDISK}/etc/init.d/cyclone

chroot ${GUESTDISK} rc-update add cyclone boot

EOL

# run the script, mounting the disk so we can create a rootfs
sudo docker run \
  -v $ROOTFSMOUNT:$GUESTDISK \
  -v $INITSCRIPT:/init.sh \
  -it --rm \
  --entrypoint sh \
  systeminit/cyclone:20231120.223123.0-sha.10c725585-dirty-amd64  \
  /init.sh

# lets go find the dev decryption key for now
sudo cp $GITROOT/lib/cyclone-server/src/dev.decryption.key $ROOTFSMOUNT

# cleanup the PACKAGEDIR
sudo umount $ROOTFSMOUNT
rm -rf $ROOTFSMOUNT $KERNELMOUNT $INITSCRIPT $KERNELISO

# make the package
#sudo tar -czvf cyclone-package.tar.gz -C $PACKAGEDIR .

sudo mv cyclone-pkg/cyclone-rootfs.ext4 /firecracker-data/rootfs.ext4

# cleanup
#sudo rm -rf $PACKAGEDIR
  ## working systeminit/cyclone:20231120.162459.0-sha.10c725585-dirty-amd64 \
  #
  #
  # built with root: 20231120.190923.0-sha.10c725585-dirty-amd64

