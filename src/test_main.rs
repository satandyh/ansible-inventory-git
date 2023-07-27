#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_git_clone_with_valid_config() {
        let repo_ssh_address = "git@example.com:user/repo.git";
        let key_path = "/path/to/ssh/key";
        let workdir = env::temp_dir();

        let result = git_clone(repo_ssh_address, key_path, &workdir.to_string_lossy());

        assert!(result.is_ok(), "Expected successful cloning");
    }

    #[test]
    #[should_panic(expected = "Error reading configuration file")]
    fn test_read_config_with_invalid_file() {
        let config_file_path = "invalid_config.yaml";
        read_config(config_file_path).unwrap();
    }

    #[test]
    #[should_panic(expected = "Error cloning repository")]
    fn test_git_clone_with_invalid_config() {
        let repo_ssh_address = "git@example.com:user/repo.git";
        let key_path = "/path/to/invalid/key";
        let workdir = env::temp_dir();

        let result = git_clone(repo_ssh_address, key_path, &workdir.to_string_lossy());

        assert!(result.is_err(), "Expected cloning to fail");
    }
}
