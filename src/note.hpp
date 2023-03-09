#include <filesystem>
#include <vector>

using std::filesystem::path;
using std::vector;

class Note {
    path _obsidian_path;
    path _hugo_path;
    vector<Note*> _backlinks;

public:
    Note(path vault, path obsidian_path, path hugo_dst);
    path getObsidianVaultPath();
    path getHugoPath();
    path getWebPath(path hugo_root);
    void addBacklink(Note* note_ref);
    vector<Note*> getBacklinks();
};
