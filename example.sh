#!/bin/sh

ex1() {
	ls | ./dirent-filter-cel \
		--name 'item' \
		--expr 'item.is_file && 0 < item.len && item.nlink >= 1 && item.mtime > 0' | tail -n 10
}

ex2() {
	find /dev -mindepth 1 -maxdepth 1 |
		./dirent-filter-cel \
			--name 'item' \
			--expr '(
				(item.is_block_device || item.is_char_device)
				&& (!(item.name.startsWith("t")))
			)' | tail -n 10
}

ex3() {
	ls -A | ./dirent-filter-cel \
		--name 'item' \
		--expr 'item.is_hidden' | tail -n 10
}

ex4() {
	find . -mindepth 1 -maxdepth 1 | ./dirent-filter-cel \
		--name 'item' \
		--expr 'item.uid == 0' | tail -n 10
}

ex5() {
	find -L /tmp -mindepth 1 -maxdepth 1 |
		./dirent-filter-cel \
			--name 'item' \
			--expr 'item.is_socket || item.is_fifo' | tail -n 10
}

ex6() {
	find . -mindepth 1 -maxdepth 1 | ./dirent-filter-cel \
		--name 'item' \
		--expr 'item.atime > item.mtime' | tail -n 10
}

ex7() {
	find . -mindepth 1 -maxdepth 1 | ./dirent-filter-cel \
		--name 'item' \
		--expr 'item.len > parseSize("0.5KiB")' | tail -n 10
}

run_example() {
	case "$1" in
	1)
		echo "Running ex1 (filter for files with at least one link and recent modification - last 10 entries)"
		ex1
		;;
	2)
		echo "Running ex2 (filter for device files in /dev, excluding those starting with 't' - last 10 entries)"
		ex2
		;;
	3)
		echo "Running ex3 (filter for hidden files - last 10 entries)"
		ex3
		;;
	4)
		echo "Running ex4 (filter for files owned by root - last 10 entries)"
		ex4
		;;
	5)
		echo "Running ex5 (filter for sockets and fifos in /tmp - last 10 entries)"
		ex5
		;;
	6)
		echo "Running ex6 (filter for files accessed after being modified - last 10 entries)"
		ex6
		;;
	7)
		echo "Running ex7 (filter for files larger than 0.5KB - last 10 entries)"
		ex7
		;;
	*)
		echo "Usage: $0 [1-7]"
		echo "Running all examples."
		echo ""
		run_example 1
		echo ""
		run_example 2
		echo ""
		run_example 3
		echo ""
		run_example 4
		echo ""
		run_example 5
		echo ""
		run_example 6
		echo ""
		run_example 7
		;;
	esac
}

if [ -z "$1" ]; then
	run_example all
else
	run_example "$1"
fi
