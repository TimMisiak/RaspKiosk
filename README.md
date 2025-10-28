# RaspKiosk

This is a simple tauri app that can be used as a full screen "kiosk" on a raspberry pi. To set this up, put the AppImage file somewhere on the raspberry pi, like `/usr/bin/raspkiosk.AppImage`. Create a kioskconfig.yaml file that indicates what page to load on startup:

```yaml
start_url: "https://example.com"
```

Then add a service config, e.g. at `sudo nano /etc/systemd/system/kiosk.service`.

```
[Unit]
Description=RaspKiosk
After=labwc.service graphical.target

[Service]
User=1000
Environment=XDG_RUNTIME_DIR=/run/user/1000
Environment=WAYLAND_DISPLAY=wayland-0
Environment=DISPLAY=:0
ExecStart=/usr/bin/raspkiosk.AppImage /usr/bin/kioskconfig.yaml
Restart=always

[Install]
WantedBy=graphical.target
```

After that, use systemctl to enable the service:

```bash
sudo systemctl enable kiosk.service
sudo systemctl start kiosk.service
```

At this point you should be able to reboot and raspkiosk will launch on startup and show the url you specified.