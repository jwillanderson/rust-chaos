## create a vendors directory containing dependencies
mkdir vendors
cd vendors
## CSFML
git clone https://github.com/SFML/CSFML.git
cd CSFML
### - latest stable release
git checkout $(git describe --tags $(git rev-list --tags --max-count=1))
cd ..
## SFML
git clone https://github.com/SFML/SFML.git
cd SFML
### - latest stable release
git checkout $(git describe --tags $(git rev-list --tags --max-count=1))
cd ..
cd ..
