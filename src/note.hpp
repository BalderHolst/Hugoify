#ifndef NOTE_INCLUDES
#define NOTE_INCLUDES


#include <filesystem>
#include <vector>

using std::filesystem::path;
using std::vector;

class Note {
    path _relative_path;
    path _hugo_path;
    vector<Note*> _backlinks;

public:
    Note(path vault, path obsidian_path);
    path getRelativePath();
    void addBacklink(Note* note_ref);
    vector<Note*> getBacklinks();
};
#endif // !NOTE_INCLUDES
