{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-23.05";
  # inputs.webtarot.url = "github:mmai/webtarot";
  inputs.webtarot.url = "..";

  outputs = { self, nixpkgs, webtarot }: 
   {
    nixosConfigurations = {

      container = nixpkgs.lib.nixosSystem {
        system = "x86_64-linux";

        modules = [
          webtarot.nixosModule
          ( { pkgs, ... }: 
          let 
            hostname = "webtarot";
          in {
            boot.isContainer = true;

            # Let 'nixos-version --json' know about the Git revision
            # of this flake.
            system.configurationRevision = nixpkgs.lib.mkIf (self ? rev) self.rev;
            system.stateVersion = "23.05";

            nixpkgs.config.permittedInsecurePackages = [ 
            "openssl-1.1.1v" 
            ];
            # Network configuration.
            networking.useDHCP = false;
            networking.firewall.allowedTCPPorts = [ 80 ];
            networking.hostName = hostname;

            nixpkgs.overlays = [ webtarot.overlay ];

            services.webtarot = {
              enable = true;
              protocol = "http";
              hostname = hostname;
            };
            users.users.webtarot.isSystemUser = true;

            environment.systemPackages = with pkgs; [ neovim ];
          })
        ];
      };

    };
  };
}
