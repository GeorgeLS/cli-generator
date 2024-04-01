#include "cli.h"

int main(int argc, char* args[]) {
	Cli cli = Cli::parse(argc, args);
    cli.print_debug();
	return 0;
}
