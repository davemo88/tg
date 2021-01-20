import { NativeModules } from 'react-native';
const { PlayerWalletModule } = NativeModules
interface PlayerWalletInterface {
   hello_react(): void;
}
export default PlayerWalletModule as PlayerWalletInterface;
