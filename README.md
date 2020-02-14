# viewerator
Console based Minerator viewer written in Rust

![Screen Shot](sample_screen.png)

Minerator is a miner application for the FPGA based altcoin mining.  It provides a web based status which can be viewed on port 80
of the host which has the FPGA cards plugged into it.  For myself, I tend to ssh into a remote host and prefer the idea of a console
based status app so I only need on remote tunnel to both view and manipulate the host machine.

Viewerator is a console app which you can install in your /usr/local/bin directory and call when you ssh into your host. (It can be installed
anywhere as it is stand alone app).  

It is a work in progress and with all the normal caveats, use it and see if it is useful to you.