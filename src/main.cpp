#include <filesystem>
#include <iostream>
#include <string>
#include <vector>
#include <fstream>

using std::string;

// Copy pasted code... 
string read_file(std::string path) {
    constexpr auto read_size = std::size_t(4096);
    auto stream = std::ifstream(path.data());
    stream.exceptions(std::ios_base::badbit);
    
    auto out = std::string();
    auto buf = std::string(read_size, '\0');
    while (stream.read(& buf[0], read_size)) {
        out.append(buf, 0, stream.gcount());
    }
    out.append(buf, 0, stream.gcount());
    return out;
}

string relative_to(string path, string relative) {
	if (relative == path.substr(0, relative.length())) {
		return path.substr(relative.length() + 1);
	}
	exit(1); // TODO
}

void convert_dir(string dir, string out_dir, string vault_root = "") {
	if (vault_root == "")
		vault_root = dir;

	for (const auto &file : std::filesystem::directory_iterator(dir)) {
		string ext = file.path().extension();

		if (file.is_directory()) {
			// Create out dir here
			convert_dir(file.path(), out_dir, vault_root);
		} else if (ext == ".md") {
			string out_file = out_dir + "/" + relative_to(file.path(), vault_root);

			/* std::ifstream myfile; myfile.open(file.path()); */
			std::cout << read_file(file.path()) << std::endl;

			std::cout << "Found note: " << out_file << std::endl;
		}
	}
}

int main(int argc, char *argv[]) {
	string vault_path = "/home/balder/Documents/uni/noter/notes";
	string out_dir = "./out";

	convert_dir(vault_path, out_dir);

	return 0;
}
