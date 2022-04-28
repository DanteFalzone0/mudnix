# source setup.sh

git config --global user.name "$GH_USERNAME"
git config --global user.email "$GH_EMAIL"

export RUSTFLAGS='-C strip=symbols'

function gitsync {
  git add --all
  git commit -m "$@"
  git push "https://$GITHUB_TOKEN@github.com/DanteFalzone0/mudnix.git" main
}

echo "Finished setup."
