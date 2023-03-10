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
    path _hugo_root;
    path _content_dir;
    std::vector<path> _excluded_paths;
    std::vector<Note> _notes;

	void _convert_dir(path dir, path out_dir, path hugo_path);
    std::vector<path> _get_excluded(); 
	string _hugoify_links(Note* note, string content);
	string _obsidian_to_hugo(Note* note);
    string _double_newlines(string content);
    string _add_header(path file_path, string contents);
    bool _is_excluded(path file_path);
    Finding _find_file(path dir, string name);
    vector<Note> _findNotes(path dir);
    Note* _getNote(path vault_path);

public:
    Converter(path vault, path hugo_root, path content_dir = "notes");
    void convert_vault(path out_dir);
    path find_file(string name);
};

#endif
