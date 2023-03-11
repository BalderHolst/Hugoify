
#include "converter.hpp"
#include "link.hpp"

#include <algorithm>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <string>
#include <vector>

using std::cout;
using std::endl;
using std::string;
using std::filesystem::path;

namespace fs = std::filesystem;


string linkify(path link_path) {
    string link;

    if (link_path.extension() == ".md") link = link_path.parent_path().string() + "/" + link_path.stem().string();
    else link = link_path.string();

    for (int i = link.length(); i >= 0 ; i--) {
        link[i] = tolower(link[i]);
        switch (link[i]) {
            case ' ': link[i] = '-'; break;
            case '(':
            case ')': link = link.substr(0, i) + link.substr(i + 1); break;
        }
    }

    return link;
}



// TODO handle path not exists
// Copy pasted code...
string read_file(std::string path) {
    constexpr auto read_size = std::size_t(4096);
    auto stream = std::ifstream(path.data());
    stream.exceptions(std::ios_base::badbit);

    auto out = std::string();
    auto buf = std::string(read_size, '\0');
    while (stream.read(&buf[0], read_size)) {
        out.append(buf, 0, stream.gcount());
    }
    out.append(buf, 0, stream.gcount());
    return out;
}

void write_file(path file_path, string contents) {
    path parent = file_path.parent_path();
    if (!fs::exists(parent)){
        fs::create_directories(parent);
    }
    
    std::ofstream myfile;
    myfile.open(file_path);
    myfile << contents;
    myfile.close();
}



int main(int argc, char *argv[]) {
	/* path vault_path = "/home/balder/Documents/uni/noter"; */
	path vault_path = "/home/balder/Documents/uni/noter-test";
	path out_dir = "/home/balder/projects/website/content/notes";
	path hugo_path = "/home/balder/projects/website";


    Converter(vault_path, hugo_path).convert_vault();
	return 0;
}
