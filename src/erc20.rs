use ethers::contract::abigen;

abigen!(
    Erc20,
    r#"[
        function name() public view virtual returns (string memory)
        function symbol() public view virtual returns (string memory)
        function decimals() public view virtual returns (uint8)
        function totalSupply() public view virtual returns (uint256)
        function balanceOf(address account) public view virtual returns (uint256)
        function transfer(address recipient, uint256 amount) public virtual returns (bool)
        function allowance(address owner, address spender) public view virtual returns (uint256)
        function approve(address spender, uint256 amount) public virtual returns (bool)
        function transferFrom(address sender, address recipient, uint256 amount) public virtual returns (bool)
        event Transfer(address indexed from, address indexed to, uint256 value)
        event Approval(address indexed owner, address indexed spender, uint256 value)
    ]"#,
    event_derives(serde::Deserialize, serde::Serialize),
);
