#https://www.raspberrypi.com/news/another-new-raspbian-release/
#raspi-config enable vnc and gl kms
SSH_TARGET="pi@raspberrypi.lan"

ssh-copy-id $SSH_TARGET
ssh $SSH_TARGET sudo apt-get install mesa-utils xorg realvnc-vnc-server
ssh $SSH_TARGET raspi-config

#then
#enable graphical target
#getty auto login
#~/.xinitrc stars