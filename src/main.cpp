
#include "converter.hpp"
#include "link.hpp"
#include "finding.hpp"

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

Finding find_file(path dir, string name) {
    name = linkify(name);
    for (const auto &file : std::filesystem::directory_iterator(dir)) {
        if (file.is_directory()) {
            Finding finding = find_file(file.path(), name);
            if (finding.was_found())
                return finding;
        } else {
            string filename = linkify(file.path().filename());

            if (filename[0] == '/') filename = filename.substr(1);

            if (filename == name) {
                return Finding(file.path());
            }
        }
    }
    return Finding("");
}


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

void write_file(string path, string contents) {
    std::ofstream myfile;
    myfile.open(path);
    myfile << contents;
    myfile.close();
}


path find_file_in_vault(path vault, string name) {

    name = ((path) name).filename().string(); // Isolate filename

    Finding finding = find_file(vault, name);

    if (finding.was_found()) {
        return ((path) finding.get_finding()).lexically_relative(vault);
    } else {
        cout << "WARNING: coud not find link \'" << name << "\'" << endl; 

        return ""; // TODO Make this not create a link.
    }
}


int main(int argc, char *argv[]) {
	path vault_path = "/home/balder/Documents/uni/noter";
	path out_dir = "/home/balder/projects/website/content/notes";
    path hugo_path = "/notes";


    Converter c = Converter(vault_path);
    c.convert_vault(out_dir, hugo_path);
	return 0;
}
