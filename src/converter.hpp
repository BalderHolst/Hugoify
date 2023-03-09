#ifndef CONVERTER_INCLUDES
#define CONVERTER_INCLUDES

#include "finding.hpp"
#include "note.hpp"

#include <vector>
#include <filesystem>

using std::string;
using std::filesystem::path;

class Converter {
    path _vault;
    std::vector<path> _excluded_paths;
    std::vector<Note> _notes;

	void _convert_dir(path dir, path out_dir, path hugo_path);
    std::vector<path> _get_excluded(); 
	string _hugoify_links(path file_path, path hugo_path, string content);
	string _obsidian_to_hugo(path file_path, path hugo_path, string content);
    string _double_newlines(string content);
    string _add_header(path file_path, string contents);
    bool _is_excluded(path file_path);
    Finding _find_file(path dir, string name);
    vector<Note> _findNotes(path dir);

public:
    void convert_vault(path out_dir, path hugo_path);
    Converter(path vault);
    path find_file(string name);
};

#endif
