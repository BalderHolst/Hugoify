// NOT USED

#include <vector>
#include <iostream>


class ObsidianLink {
    public:
    int start;
    int end;
    int path;
    int name;
    bool shown;
};

class ScrapedNote {
    public:
    std::vector<ObsidianLink> links;
};
    
