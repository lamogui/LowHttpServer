LowHttpServer
=============

Low-tech http server in rust to anwser GET http requests as fast as possible using pre-packed answers compiled with LowHttpPack.

How-to use it with a basic VPS ubuntu server
--------------------------------------------
On your desktop (I use windows here so find linux equivalents):
  * Pack your website directory with [LowHttpPack](https://github.com/lamogui/LowHttpPack) using ```./LowHttpPack <path/to/pack>```
  * Transfert your website.pack using [putty pscp](https://www.putty.org/) ```pscp <username>@<yourwebsiteip>:</target/path/to/your/website.pack> website.pack```

On your VPS using putty:
 * Get the source code via git ```git clone https://github.com/lamogui/LowHttpServer```
 * Compile the server using ```cargo build --release```
 * Go to ```cd /etc/systemd/system/```
 * Create a file for the service ```sudo nano lowhttpserver.service``` you can use one of the server as reference just replace the absolute paths by yours
 * Reload the systemd files ```sudo systemctl daemon-reload``` (do that each time you edit the .service file)
 * Start / Enable at each restart your server ```sudo systemctl restart lowhttpserver.service``` and/or ```sudo systemctl enable lowhttpserver.service```
 * You can check erverything goes well using ```sudo systemctl status lowhttpserver.service```
 * Enjoy !

 

 
