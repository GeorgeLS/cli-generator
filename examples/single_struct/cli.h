#ifndef _CLI_H_
#define _CLI_H_

#include <cstdint>
#include <cstdlib>
#include <cstring>
#include <cstdio>
#include <cerrno>
#include <string>
#include <vector>

struct Cli {
    bool some;
    bool verbose;
    int16_t param;
    float float_value;
    std::string str;
    std::vector<uint32_t> many_values;

    void print_debug() {
        printf("Cli {\n");
        printf("\tsome: %s\n", this->some ? "true" : "false");
        printf("\tverbose: %s\n", this->verbose ? "true" : "false");
        printf("\tparam: %d\n", this->param);
        printf("\tfloat_value: %f\n", this->float_value);
        printf("\tstr: %s\n", this->str.c_str());
        printf("\tmany_values: [\n");
        for (size_t i = 0; i != this->many_values.size(); ++i) {
            printf("\t%d,\n", this->many_values[i]);
        }
        printf("\t]\n");
        printf("}\n");
    }

    static void help() {
        printf("Usage: Cli [OPTIONS]\n"
        "\n"
        "Options:\n"
        "    -h, --help\n"
        "    -s, --some\n"
        "    -v, --verbose\n"
        "    -p, --param <PARAM>\n"
        "    -f, --float-value <FLOAT_VALUE>\n"
        "    --str <STR>\n"
        "    -m, --many-values <MANY_VALUES>\n"
    );
    exit(0);
}

    static bool is_option(char* arg) {
        static const char* valid_options[] = {
            "-s",
            "--some",
            "-v",
            "--verbose",
            "-p",
            "--param",
            "-f",
            "--float-value",
            "--str",
            "-m",
            "--many-values",
        };

        for (size_t i = 0; i != 11; ++i) {
            if (strcmp(arg, valid_options[i]) == 0) {
                return true;
            }
        }

        return false;
    }

    static Cli parse (int argc, char *args[]) {
        --argc;
        ++args;

        const char* mandatory_field_names[] = { "some", "verbose", "param", "float_value", "str", "many_values", };
        bool mandatory_fields_seen[sizeof(mandatory_field_names)/sizeof(mandatory_field_names[0])] = { false };

        Cli res = {};
        for (int i = 0; i != argc; ++i, ++args) {
            char *arg = args[0];
            if (strcmp("-h", arg) == 0 || strcmp("--help", arg) == 0) {
                Cli::help();
            } else if (strcmp(arg, "-s") == 0 || strcmp(arg, "--some") == 0) {
                bool arg_res = true;
                res.some = arg_res;
                mandatory_fields_seen[0] = true;
            } else if (strcmp(arg, "-v") == 0 || strcmp(arg, "--verbose") == 0) {
                bool arg_res = true;
                res.verbose = arg_res;
                mandatory_fields_seen[1] = true;
            } else if (strcmp(arg, "-p") == 0 || strcmp(arg, "--param") == 0 || strcmp(arg, "--omg") == 0) {
                ++args;
                ++i;
                if (i == argc || Cli::is_option(args[0])) {
                    printf("Expected value for option '%s' but no value was provided", arg);
                    exit(1);
                }
                char* arg_value = args[0];
                int16_t arg_res = static_cast<int16_t>(std::strtoll(arg_value, nullptr, 10));

                if (errno == ERANGE) {
                    printf("Value '%s' of option '%s' out of range for integer type", arg_value, arg);
                    exit(1);
                }
                if (arg_res == 0 && strcmp(arg, "0") != 0) {
                    printf("Value '%s' of option '%s' is not a valid integer", arg_value, arg);
                    exit(1);
                }
                res.param = arg_res;
                mandatory_fields_seen[2] = true;
            } else if (strcmp(arg, "-f") == 0 || strcmp(arg, "--float-value") == 0) {
                ++args;
                ++i;
                if (i == argc || Cli::is_option(args[0])) {
                    printf("Expected value for option '%s' but no value was provided", arg);
                    exit(1);
                }
                char* arg_value = args[0];
                float arg_res = static_cast<float>(std::strtof(arg_value, nullptr));

                if (errno == ERANGE) {
                    printf("Value '%s' of option '%s' out of range for integer type", arg_value, arg);
                    exit(1);
                }
                if (arg_res == 0 && strcmp(arg, "0") != 0) {
                    printf("Value '%s' of option '%s' is not a valid integer", arg_value, arg);
                    exit(1);
                }
                res.float_value = arg_res;
                mandatory_fields_seen[3] = true;
            } else if (strcmp(arg, "--str") == 0) {
                ++args;
                ++i;
                if (i == argc) {
                    printf("Expected value for option '%s' but no value was provided", arg);
                    exit(1);
                }
                std::string arg_res = args[0];
                res.str = arg_res;
                mandatory_fields_seen[4] = true;
            } else if (strcmp(arg, "-m") == 0 || strcmp(arg, "--many-values") == 0) {
                ++args;
                ++i;
                if (i == argc || Cli::is_option(args[0])) {
                    printf("Expected value for option '%s' but no value was provided", arg);
                    exit(1);
                }
                char* arg_value = args[0];
                uint32_t arg_res = static_cast<uint32_t>(std::strtoll(arg_value, nullptr, 10));

                if (errno == ERANGE) {
                    printf("Value '%s' of option '%s' out of range for integer type", arg_value, arg);
                    exit(1);
                }
                if (arg_res == 0 && strcmp(arg, "0") != 0) {
                    printf("Value '%s' of option '%s' is not a valid integer", arg_value, arg);
                    exit(1);
                }
                res.many_values.push_back(arg_res);
                mandatory_fields_seen[5] = true;
            } else {
                printf("Unknown option '%s'\n", arg);
                exit(1);
            }
        }

        bool not_seen_any = false;
        for (size_t i = 0; i != sizeof(mandatory_field_names)/sizeof(mandatory_field_names[0]); ++i) {
            if (!mandatory_fields_seen[i]) {
                printf("--%s was required but it was not provided\n", mandatory_field_names[i]);
                not_seen_any = true;
            }
        }
        if (not_seen_any) {
            exit(1);
        }
        return res;
    }
};

#endif // _CLI_H_
