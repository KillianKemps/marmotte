# Marmotte

This Gopher client written in Rust will be kept minimalistic.

## Installation
Simply run this command
```
make install
```

## Usage
```
marmotte [url]
```

Available commands:
```
get [url]: Get this url
[index]: Follow link index
up: Go up one directory
back: Go back previous page
bk: List bookmarks
bk [index]: Follow bookmark
bk add: Add bookmark
bk rm: Remove bookmark
quit: Quit this program
```

Example:
```
$ marmotte
Welcome to marmotte!
Enter 'help' if you don't know how to start. Have a nice journey in the Gopherspace!

marmotte> go zaibatsu.circumlunar.space
		
		           MARE   TRANQUILLITATIS   PEOPLE'S   CIRCUMLUNAR
		    .       _____   _    ___ ____    _  _____ ____  _   _    '
		           |__  /  / \  |_ _| __ )  / \|_   _/ ___|| | | |
		             / /  / _ \  | ||  _ \ / _ \ | | \___ \| | | |
		        *   / /_ / ___ \ | || |_) / ___ \| |  ___) | |_| |
		           /____/_/   \_\___|____/_/   \_\_| |____/ \___/   .
		
		                Flagship Circumlunar Colony Est. 2018
[…]
```

## Project features:
- [x] Handle text resources (type 0)
- [x] Handle Gopher menus (type 1)
- [ ] Handle Gopher search (type 7)
- [x] Go one directory up from the path (`up` command)
- [x] Go back several pages before in history (`back`command)
- [x] Save links into bookmarks
- [ ] Follow links in text/markdown resources
- [ ] Cache pages by default for fast and offline browsing
- [ ] Connect with SSL/TLS when the remote server implements it
