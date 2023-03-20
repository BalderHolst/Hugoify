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

	void _convertDir(path dir, path out_dir, path hugo_path);
    std::vector<path> _getExcluded(); 
    bool _isExcluded(path file_path);
    Finding _findFile(path dir, string name);
    vector<Note> _findNotes(path dir);
    Note* _getNote(path vault_path);
    string _obsidianToHugo(Note* note);
	void _hugoifyLinks(Note* note, string& content);
    void _addBacklinks(Note* note);
    void _formatLatex(string& content);
    void _formatCboxes(string& content);
    vector<string> _extractTags(string& content);
    void _addHeader(path file_path, vector<string> tags, string& contents);
    void _addIndexFile();

public:
    Converter(path vault, path hugo_root, path content_dir = "notes");
    void convertVault();
    path findFile(string name);
};

#endif
