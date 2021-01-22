import { NativeModules } from 'react-native';
const { PlayerWalletModule } = NativeModules
interface PlayerWalletInterface {
   call_cli(string): void;
}
export default PlayerWalletModule as PlayerWalletInterface;
