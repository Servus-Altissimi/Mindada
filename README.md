
# Mindada, 64 bit research OS

```
                               _______                _______                 
 __  __   ___   .--.   _..._   \  ___ `'.             \  ___ `'.              
|  |/  `.'   `. |__| .'     '.  ' |--.\  \             ' |--.\  \             
|   .-.  .-.   '.--..   .-.   . | |    \  '            | |    \  '            
|  |  |  |  |  ||  ||  '   '  | | |     |  '    __     | |     |  '    __     
|  |  |  |  |  ||  ||  |   |  | | |     |  | .:--.'.   | |     |  | .:--.'.   
|  |  |  |  |  ||  ||  |   |  | | |     ' .'/ |   \ |  | |     ' .'/ |   \ |  
|  |  |  |  |  ||  ||  |   |  | | |___.' /' `" __ | |  | |___.' /' `" __ | |  
|__|  |__|  |__||__||  |   |  |/_______.'/   .'.''| | /_______.'/   .'.''| |  
                    |  |   |  |\_______|/   / /   | |_\_______|/   / /   | |_ 
                    |  |   |  |             \ \._,\ '/             \ \._,\ '/ 
                    '--'   '--'              `--'  `"               `--'  `"

                                                  Romanized Arabic for workbench.
```

## Goal & Purpose
I'm writing this OS to compliment my thessis on adaptive bandwidth and buffering strategies on network performance in simulated agent swarms. Existing operating systems introduce overhead and lack fine-grained control over buffer allocation and bandwidth scheduling. A lightweight custom OS allows precise modeling of adaptive strategies in a controlled simulation environment. It's also is a great thing to have on my resume. 

Right now it's a framework. It doesn't do a lot right now besides ping 8.8.8.8.

## Choice of technology
I use Rust for this OS instead of C++ (like OSdev recommends), for multiple reasons. It's also a challenge for me in ways. Having to write my own, everything, is far different then relying on the existing ecosystem Rust has. It also, again, looks good on my resume to say I wrote an OS with Rust. (I'm so getting those damn goverment contracts). Most of the current code is based on existing C code I translated into Rust. I don't follow [OSdev.wiki](https://osdev.wiki). Not because I believe I'm far too good for it, but because that goes against the point of this OS. Being an unique research OS with fine grained network control. 

## To-Do
Goals to reach are:
- Fine grained network control
- Simulate multiple network agents
- Log results into external system
- Recognize user input

Small contributions are welcome \('o')/